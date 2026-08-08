//! Enrichment orchestration — the deterministic core of the metadata pipeline.
//!
//! For each file: read its embedded signals, fetch canonical candidates online (by ISBN if we
//! have one, else by title/author), score the best candidate, and — if confidence clears the
//! gate — write the canonical metadata into the file and rename it. Below the gate, the file is
//! left untouched and reported as low-confidence (the seam where an agentic disambiguation step
//! will later plug in). Dry-run by default, like `rename`.

use crate::error::Result;
use crate::matcher::score;
use crate::metadata::{extract_metadata, to_lastname_first, EpubMetadata};
use crate::provider::{openlibrary_by_isbn, openlibrary_search, BookRecord};
use crate::rename::{generate_filename, RenameOptions};
use crate::writer::{write_metadata, WriteConfig};
use std::path::{Path, PathBuf};

pub struct EnrichConfig {
    pub min_confidence: f32,
    pub pattern: String,
    pub rename_opts: RenameOptions,
    pub write: WriteConfig,
    /// Move enriched files here (e.g. the CWA ingest folder); None = rename in place.
    pub out_dir: Option<PathBuf>,
    pub execute: bool,
}

#[derive(Debug, PartialEq)]
pub enum EnrichStatus {
    Applied,        // metadata written + renamed/moved
    Preview,        // dry-run: would apply
    LowConfidence,  // best match below the gate — left untouched
    NoMatch,        // provider returned nothing
    NoSignals,      // file had neither title/author nor ISBN to search on
}

pub struct EnrichOutcome {
    pub path: PathBuf,
    pub status: EnrichStatus,
    pub confidence: f32,
    pub record: Option<BookRecord>,
    pub new_name: Option<String>,
}

pub fn enrich_file(path: &Path, cfg: &EnrichConfig) -> Result<EnrichOutcome> {
    let local = extract_metadata(path)?;

    if !local.has_metadata() && local.isbn.is_none() {
        return Ok(outcome(path, EnrichStatus::NoSignals, 0.0, None, None));
    }

    let (candidates, by_isbn) = fetch_candidates(&local)?;
    let best = candidates.into_iter()
        .map(|c| (score(&local, &c, by_isbn), c))
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let (conf, rec) = match best {
        Some(x) => x,
        None => return Ok(outcome(path, EnrichStatus::NoMatch, 0.0, None, None)),
    };

    if conf < cfg.min_confidence {
        return Ok(outcome(path, EnrichStatus::LowConfidence, conf, Some(rec), None));
    }

    let derived = record_to_metadata(&rec, path);
    let new_name = generate_filename(&derived, &cfg.pattern, &cfg.rename_opts);

    if !cfg.execute {
        return Ok(outcome(path, EnrichStatus::Preview, conf, Some(rec), Some(new_name)));
    }

    write_metadata(path, &rec, &cfg.write)?;

    let dest_dir = cfg.out_dir.clone()
        .or_else(|| path.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    std::fs::create_dir_all(&dest_dir).ok();
    let dest = dest_dir.join(&new_name);
    if dest != path {
        std::fs::rename(path, &dest)?;
    }
    Ok(outcome(&dest, EnrichStatus::Applied, conf, Some(rec), Some(new_name)))
}

/// ISBN first (authoritative), then fall back to a title/author search.
fn fetch_candidates(local: &EpubMetadata) -> Result<(Vec<BookRecord>, bool)> {
    if let Some(isbn) = &local.isbn {
        let by = openlibrary_by_isbn(isbn)?;
        if !by.is_empty() {
            return Ok((by, true));
        }
    }
    if let Some(title) = &local.title {
        // local author is stored "Last, First"; search wants natural order
        let author_q = local.author.as_ref().map(|a| a.replace(", ", " "));
        return Ok((openlibrary_search(title, author_q.as_deref())?, false));
    }
    Ok((Vec::new(), false))
}

fn record_to_metadata(rec: &BookRecord, path: &Path) -> EpubMetadata {
    EpubMetadata {
        source_path: path.to_path_buf(),
        title: rec.title.clone(),
        author: rec.authors.first().map(|a| to_lastname_first(a)),
        publisher: rec.publisher.clone(),
        date: rec.date.clone(),
        year: rec.year.clone(),
        isbn: rec.isbn.clone(),
        language: rec.language.clone(),
    }
}

fn outcome(path: &Path, status: EnrichStatus, confidence: f32,
           record: Option<BookRecord>, new_name: Option<String>) -> EnrichOutcome {
    EnrichOutcome { path: path.to_path_buf(), status, confidence, record, new_name }
}
