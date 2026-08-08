//! Confidence scoring — how sure are we that a fetched record is the same book as the local
//! file? This is the deterministic gate that stands in for the (future) agentic disambiguation:
//! an ISBN hit is authoritative; a title/author search is scored on normalized similarity, and
//! anything below the caller's threshold is flagged for review rather than silently applied.

use crate::metadata::EpubMetadata;
use crate::provider::BookRecord;
use strsim::jaro_winkler;

/// Normalize free text for comparison: lowercase, strip punctuation, collapse whitespace.
fn norm(s: &str) -> String {
    let cleaned: String = s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize a person name so token *order* doesn't matter ("Austen, Jane" ~ "Jane Austen").
fn norm_name(s: &str) -> String {
    let mut toks: Vec<String> = norm(s).split_whitespace().map(str::to_string).collect();
    toks.sort();
    toks.join(" ")
}

/// Confidence in [0,1] that `cand` describes the same book as `local`.
/// `by_isbn` = the candidate was fetched via the file's ISBN (authoritative).
pub fn score(local: &EpubMetadata, cand: &BookRecord, by_isbn: bool) -> f32 {
    let title_sim = match (&local.title, &cand.title) {
        (Some(a), Some(b)) => jaro_winkler(&norm(a), &norm(b)) as f32,
        _ => 0.5, // unknown local title → neutral, don't punish or reward
    };

    if by_isbn {
        // ISBN is a hard key; title agreement only nudges it.
        return (0.9 + 0.1 * title_sim).min(1.0);
    }

    let author_sim = match (&local.author, cand.authors.first()) {
        (Some(a), Some(b)) => jaro_winkler(&norm_name(a), &norm_name(b)) as f32,
        (None, _) => 0.5,
        (Some(_), None) => 0.0,
    };
    0.6 * title_sim + 0.4 * author_sim
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn local(title: &str, author: &str) -> EpubMetadata {
        EpubMetadata {
            source_path: PathBuf::from("x.epub"),
            title: Some(title.into()), author: Some(author.into()),
            publisher: None, date: None, year: None, isbn: None, language: None,
        }
    }
    fn cand(title: &str, author: &str) -> BookRecord {
        BookRecord { title: Some(title.into()), authors: vec![author.into()], ..Default::default() }
    }

    #[test]
    fn exact_match_scores_high() {
        assert!(score(&local("Dune", "Herbert, Frank"), &cand("Dune", "Frank Herbert"), false) > 0.95);
    }

    #[test]
    fn wrong_book_scores_low() {
        assert!(score(&local("Dune", "Herbert, Frank"), &cand("Neuromancer", "William Gibson"), false) < 0.6);
    }

    #[test]
    fn isbn_match_is_authoritative() {
        assert!(score(&local("dune", "x"), &cand("Dune", "y"), true) >= 0.9);
    }
}
