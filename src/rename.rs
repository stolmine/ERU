use crate::error::Result;
use crate::metadata::EpubMetadata;
use std::path::PathBuf;

pub struct RenameAction {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Default)]
pub struct RenameOptions {
    pub space_char: Option<char>,
    pub no_comma: bool,
}

pub fn generate_filename(metadata: &EpubMetadata, pattern: &str, opts: &RenameOptions) -> String {
    let mut result = String::with_capacity(pattern.len() + 32);
    let mut remaining = pattern;

    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix("{author}") {
            if let Some(author) = &metadata.author {
                let value = if opts.no_comma { author.replace(", ", " ") } else { author.clone() };
                result.push_str(&apply_space_char(&value, opts.space_char));
            }
            remaining = rest;
        } else if let Some((placeholder, value)) = try_match_placeholder(remaining, metadata) {
            if let Some(v) = value {
                result.push_str(&apply_space_char(v, opts.space_char));
            }
            remaining = &remaining[placeholder.len()..];
        } else if let Some(c) = remaining.chars().next() {
            let c = match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                ' ' if opts.space_char.is_some() => opts.space_char.unwrap(),
                _ => c,
            };
            result.push(c);
            remaining = &remaining[c.len_utf8()..];
        }
    }

    let result = cleanup_filename(&result);
    format!("{}.epub", result)
}

fn try_match_placeholder<'a>(s: &'a str, m: &'a EpubMetadata) -> Option<(&'static str, Option<&'a str>)> {
    const PLACEHOLDERS: [(&str, fn(&EpubMetadata) -> &Option<String>); 6] = [
        ("{title}", |m| &m.title),
        ("{publisher}", |m| &m.publisher),
        ("{date}", |m| &m.date),
        ("{year}", |m| &m.year),
        ("{isbn}", |m| &m.isbn),
        ("{language}", |m| &m.language),
    ];
    for &(ph, getter) in &PLACEHOLDERS {
        if s.starts_with(ph) {
            return Some((ph, getter(m).as_deref()));
        }
    }
    None
}

fn apply_space_char(value: &str, space_char: Option<char>) -> String {
    match space_char {
        Some(sc) => value.replace(' ', &sc.to_string()),
        None => value.to_string(),
    }
}

fn cleanup_filename(s: &str) -> String {
    let mut result = s.to_string();
    // Collapse multiple spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    // Remove orphaned separators (e.g., " - " at start/end, or "- -")
    result = result.trim().to_string();
    result = result.trim_start_matches(|c| c == '-' || c == ' ').to_string();
    result = result.trim_end_matches(|c| c == '-' || c == ' ').to_string();
    // Clean up patterns like " - " at edges or "- -"
    while result.starts_with("- ") { result = result[2..].to_string(); }
    while result.ends_with(" -") { result = result[..result.len()-2].to_string(); }
    result = result.replace(" - - ", " - ");
    result = result.replace("  ", " ");
    result.trim().to_string()
}

pub fn create_rename_action(metadata: &EpubMetadata, pattern: &str, opts: &RenameOptions) -> RenameAction {
    let new_filename = generate_filename(metadata, pattern, opts);
    let from = metadata.source_path.clone();
    let to = from
        .parent()
        .map(|p| p.join(&new_filename))
        .unwrap_or_else(|| PathBuf::from(&new_filename));

    RenameAction { from, to }
}

pub fn execute_rename(action: &RenameAction) -> Result<()> {
    std::fs::rename(&action.from, &action.to)?;
    Ok(())
}
