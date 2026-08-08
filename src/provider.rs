//! Online metadata providers. Given a file's local signals (embedded metadata / ISBN), fetch
//! candidate canonical records to enrich from. Open Library is the primary backend: fully
//! keyless, resolves author names, ISBNs, year, publisher, subjects, and a cover id.

use crate::error::{EruError, Result};
use serde_json::Value;

/// A canonical book record from an online provider — the source we enrich a file toward.
#[derive(Debug, Clone, Default)]
pub struct BookRecord {
    pub title: Option<String>,
    pub authors: Vec<String>,   // natural "First Last" order, as the provider gives them
    pub publisher: Option<String>,
    pub date: Option<String>,   // best available publication date (often just a year)
    pub year: Option<String>,
    pub isbn: Option<String>,   // preferred: ISBN-13, else ISBN-10
    pub language: Option<String>,
    pub subjects: Vec<String>,
    pub cover_url: Option<String>,
    pub source: &'static str,
}

const OL_SEARCH: &str = "https://openlibrary.org/search.json";
const OL_FIELDS: &str = "title,author_name,first_publish_year,isbn,publisher,cover_i,subject,language";
const UA: &str = "ERU/0.1 (+https://github.com/stolmine/ERU; ebook metadata enricher)";

fn get_json(url: &str) -> Result<Value> {
    ureq::get(url)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| EruError::Network(e.to_string()))?
        .into_json::<Value>()
        .map_err(|e| EruError::Provider(e.to_string()))
}

/// Look up candidates by ISBN (authoritative — the caller scores these highly).
pub fn openlibrary_by_isbn(isbn: &str) -> Result<Vec<BookRecord>> {
    let clean: String = isbn.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let url = format!("{OL_SEARCH}?isbn={clean}&fields={OL_FIELDS}&limit=5");
    parse_ol_docs(&get_json(&url)?)
}

/// Fuzzy search by title (and author, if known).
pub fn openlibrary_search(title: &str, author: Option<&str>) -> Result<Vec<BookRecord>> {
    let mut url = format!("{OL_SEARCH}?title={}&fields={OL_FIELDS}&limit=5", urlencode(title));
    if let Some(a) = author {
        url.push_str(&format!("&author={}", urlencode(a)));
    }
    parse_ol_docs(&get_json(&url)?)
}

fn parse_ol_docs(v: &Value) -> Result<Vec<BookRecord>> {
    let docs = v.get("docs").and_then(|d| d.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::with_capacity(docs.len());
    for d in &docs {
        let title = str_field(d, "title");
        let authors = arr_strings(d, "author_name");
        let year = d.get("first_publish_year").and_then(Value::as_i64).map(|y| y.to_string());
        let isbn = d.get("isbn").and_then(Value::as_array).and_then(|a| pick_isbn(a));
        let publisher = first_of(d, "publisher");
        let language = first_of(d, "language");
        let subjects = arr_strings(d, "subject").into_iter().take(8).collect();
        let cover_url = d.get("cover_i").and_then(Value::as_i64)
            .map(|id| format!("https://covers.openlibrary.org/b/id/{id}-L.jpg"));
        out.push(BookRecord {
            title, authors, publisher, date: year.clone(), year,
            isbn, language, subjects, cover_url, source: "openlibrary",
        });
    }
    Ok(out)
}

fn str_field(d: &Value, k: &str) -> Option<String> {
    d.get(k).and_then(Value::as_str).map(str::to_string)
}

fn arr_strings(d: &Value, k: &str) -> Vec<String> {
    d.get(k).and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn first_of(d: &Value, k: &str) -> Option<String> {
    d.get(k).and_then(Value::as_array).and_then(|a| a.first())
        .and_then(Value::as_str).map(str::to_string)
}

/// Prefer an ISBN-13, else ISBN-10.
fn pick_isbn(arr: &[Value]) -> Option<String> {
    let isbns: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
    isbns.iter().find(|s| s.len() == 13)
        .or_else(|| isbns.iter().find(|s| s.len() == 10))
        .map(|s| s.to_string())
}

/// Minimal, byte-wise percent-encoding (form style: space -> '+'), UTF-8 safe.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b' ' => out.push('+'),
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencode("dune herbert"), "dune+herbert");
        assert_eq!(urlencode("A&B"), "A%26B");
    }

    #[test]
    fn test_pick_isbn_prefers_13() {
        let arr = vec![Value::from("0441013597"), Value::from("9780441013593")];
        assert_eq!(pick_isbn(&arr), Some("9780441013593".to_string()));
    }
}
