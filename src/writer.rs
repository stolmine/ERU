//! Metadata writer — shells out to Calibre's `ebook-meta` to write canonical fields into the
//! book file. Calibre isn't a library dependency; the command is configurable so it can be the
//! host binary (`ebook-meta`) or a container invocation
//! (`docker exec -i calibre-web-automated ebook-meta`). When the tool runs in a container, a
//! host:container path prefix map rewrites the file path so the container sees it.

use crate::error::{EruError, Result};
use crate::provider::BookRecord;
use std::path::Path;
use std::process::Command;

pub struct WriteConfig {
    /// The `ebook-meta` invocation, already split into argv (program + leading args).
    pub ebook_meta_cmd: Vec<String>,
    /// Optional (host_prefix, container_prefix) to translate the file path for a containerized tool.
    pub path_map: Option<(String, String)>,
}

impl WriteConfig {
    /// Parse a space-separated command and an optional "host:container" path map.
    pub fn new(cmd: &str, path_map: Option<&str>) -> Self {
        let ebook_meta_cmd: Vec<String> = cmd.split_whitespace().map(str::to_string).collect();
        let path_map = path_map.and_then(|m| m.split_once(':').map(|(h, c)| (h.to_string(), c.to_string())));
        WriteConfig { ebook_meta_cmd, path_map }
    }
}

/// Write the record's fields into the file at `path` via `ebook-meta`.
pub fn write_metadata(path: &Path, rec: &BookRecord, cfg: &WriteConfig) -> Result<()> {
    if cfg.ebook_meta_cmd.is_empty() {
        return Err(EruError::ExternalTool("empty ebook-meta command".into()));
    }
    let target = map_path(path, &cfg.path_map);

    let mut cmd = Command::new(&cfg.ebook_meta_cmd[0]);
    cmd.stdin(std::process::Stdio::null()); // never block on stdin (e.g. a stray `docker exec -i`)
    cmd.args(&cfg.ebook_meta_cmd[1..]);
    cmd.arg(&target);

    if let Some(t) = &rec.title {
        cmd.arg("--title").arg(t);
    }
    if !rec.authors.is_empty() {
        cmd.arg("--authors").arg(rec.authors.join(" & ")); // ebook-meta's author separator
    }
    if let Some(p) = &rec.publisher {
        cmd.arg("--publisher").arg(p);
    }
    if let Some(d) = &rec.date {
        cmd.arg("--date").arg(d);
    }
    if let Some(i) = &rec.isbn {
        cmd.arg("--isbn").arg(i);
    }
    if let Some(l) = &rec.language {
        cmd.arg("--language").arg(l);
    }
    if !rec.subjects.is_empty() {
        cmd.arg("--tags").arg(rec.subjects.join(", "));
    }

    let out = cmd.output()
        .map_err(|e| EruError::ExternalTool(format!("spawning {}: {e}", cfg.ebook_meta_cmd[0])))?;
    if !out.status.success() {
        return Err(EruError::ExternalTool(format!(
            "ebook-meta failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

fn map_path(path: &Path, map: &Option<(String, String)>) -> String {
    let s = path.to_string_lossy().to_string();
    if let Some((host, container)) = map {
        if let Some(rest) = s.strip_prefix(host) {
            return format!("{container}{rest}");
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cmd_and_map() {
        let c = WriteConfig::new("docker exec -i cwa ebook-meta", Some("/mnt/x:/data"));
        assert_eq!(c.ebook_meta_cmd, vec!["docker", "exec", "-i", "cwa", "ebook-meta"]);
        assert_eq!(c.path_map, Some(("/mnt/x".into(), "/data".into())));
    }

    #[test]
    fn maps_path_prefix() {
        let map = Some(("/mnt/x".to_string(), "/data".to_string()));
        assert_eq!(map_path(Path::new("/mnt/x/book.epub"), &map), "/data/book.epub");
        assert_eq!(map_path(Path::new("/other/book.epub"), &map), "/other/book.epub");
    }
}
