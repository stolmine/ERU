use crate::error::Result;
use chrono::NaiveDate;
use epub::doc::EpubDoc;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EpubMetadata {
    pub source_path: PathBuf,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub isbn: Option<String>,
    pub language: Option<String>,
}

impl EpubMetadata {
    pub fn has_metadata(&self) -> bool {
        self.title.is_some() || self.author.is_some()
    }
}

pub fn extract_metadata(path: &Path) -> Result<EpubMetadata> {
    let mut doc = EpubDoc::new(path)?;

    let title = get_metadata_field(&mut doc, "title");
    let author = get_metadata_field(&mut doc, "creator").map(|a| to_lastname_first(&a));
    let publisher = get_metadata_field(&mut doc, "publisher");
    let date = get_metadata_field(&mut doc, "date");
    let year = date.as_ref().and_then(|d| extract_year(d));
    let isbn = get_metadata_field(&mut doc, "identifier")
        .filter(|id| is_isbn(id));
    let language = get_metadata_field(&mut doc, "language");

    Ok(EpubMetadata {
        source_path: path.to_path_buf(),
        title,
        author,
        publisher,
        date,
        year,
        isbn,
        language,
    })
}

fn get_metadata_field(doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>, field: &str) -> Option<String> {
    doc.mdata(field)
        .map(|item| item.value.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn to_lastname_first(name: &str) -> String {
    // Already in "Lastname, Firstname" format
    if name.contains(',') {
        return name.to_string();
    }

    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => String::new(),
        1 => parts[0].to_string(),
        _ => {
            let (last, first) = parts.split_last().unwrap();
            format!("{}, {}", last, first.join(" "))
        }
    }
}

fn extract_year(date_str: &str) -> Option<String> {
    const FORMATS: &[&str] = &["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d", "%Y", "%Y-%m", "%Y/%m"];

    for &format in FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(date.format("%Y").to_string());
        }
    }

    if date_str.len() >= 4 && date_str.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
        return Some(date_str[..4].to_string());
    }

    None
}

fn is_isbn(identifier: &str) -> bool {
    let cleaned = identifier
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
        .collect::<String>();

    cleaned.len() == 10 || cleaned.len() == 13
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_year_iso_format() {
        assert_eq!(extract_year("2023-05-15"), Some("2023".to_string()));
        assert_eq!(extract_year("2020-01-01"), Some("2020".to_string()));
    }

    #[test]
    fn test_extract_year_slash_format() {
        assert_eq!(extract_year("2023/05/15"), Some("2023".to_string()));
    }

    #[test]
    fn test_extract_year_year_only() {
        assert_eq!(extract_year("2023"), Some("2023".to_string()));
    }

    #[test]
    fn test_extract_year_year_month() {
        assert_eq!(extract_year("2023-05"), Some("2023".to_string()));
    }

    #[test]
    fn test_extract_year_invalid() {
        assert_eq!(extract_year("invalid"), None);
        assert_eq!(extract_year(""), None);
    }

    #[test]
    fn test_is_isbn_valid() {
        assert!(is_isbn("978-0-123456-78-9"));
        assert!(is_isbn("9780123456789"));
        assert!(is_isbn("0-123456-78-X"));
        assert!(is_isbn("012345678X"));
    }

    #[test]
    fn test_is_isbn_invalid() {
        assert!(!is_isbn("12345"));
        assert!(!is_isbn(""));
        assert!(!is_isbn("not-an-isbn"));
    }

    #[test]
    fn test_to_lastname_first() {
        assert_eq!(to_lastname_first("Jane Smith"), "Smith, Jane");
        assert_eq!(to_lastname_first("John Ronald Reuel Tolkien"), "Tolkien, John Ronald Reuel");
        assert_eq!(to_lastname_first("Prince"), "Prince");
        assert_eq!(to_lastname_first("Smith, Jane"), "Smith, Jane");
        assert_eq!(to_lastname_first(""), "");
    }
}
