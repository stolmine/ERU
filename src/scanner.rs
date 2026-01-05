use crate::error::{EruError, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_path(path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Err(EruError::PathNotFound(path.to_path_buf()));
    }

    let mut epubs = match (path.is_file(), path.is_dir()) {
        (true, _) => {
            if is_epub(path) {
                vec![path.to_path_buf()]
            } else {
                return Err(EruError::NotAnEpub(path.to_path_buf()));
            }
        }
        (_, true) => scan_directory(path, recursive)?,
        _ => return Err(EruError::InvalidPath(path.to_path_buf())),
    };

    epubs.sort_unstable();
    Ok(epubs)
}

fn scan_directory(path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let depth = if recursive { usize::MAX } else { 1 };

    let epubs = WalkDir::new(path)
        .max_depth(depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_epub(e.path()))
        .map(|e| e.into_path())
        .collect();

    Ok(epubs)
}

fn is_epub(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("epub"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_epub() {
        assert!(is_epub(Path::new("book.epub")));
        assert!(is_epub(Path::new("book.EPUB")));
        assert!(is_epub(Path::new("book.ePub")));
        assert!(!is_epub(Path::new("book.pdf")));
        assert!(!is_epub(Path::new("book")));
    }

    #[test]
    fn test_scan_file_epub() {
        let temp_dir = tempfile::tempdir().unwrap();
        let epub_path = temp_dir.path().join("test.epub");
        fs::write(&epub_path, b"test").unwrap();

        let result = scan_path(&epub_path, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], epub_path);
    }

    #[test]
    fn test_scan_file_not_epub() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        fs::write(&pdf_path, b"test").unwrap();

        let result = scan_path(&pdf_path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_directory_immediate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();

        fs::write(dir.join("book1.epub"), b"test").unwrap();
        fs::write(dir.join("book2.EPUB"), b"test").unwrap();
        fs::write(dir.join("ignore.pdf"), b"test").unwrap();

        let mut result = scan_path(dir, false).unwrap();
        result.sort();

        assert_eq!(result.len(), 2);
        assert!(result[0].ends_with("book1.epub"));
        assert!(result[1].ends_with("book2.EPUB"));
    }

    #[test]
    fn test_scan_directory_recursive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();
        let subdir = dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        fs::write(dir.join("root.epub"), b"test").unwrap();
        fs::write(subdir.join("nested.epub"), b"test").unwrap();
        fs::write(dir.join("ignore.txt"), b"test").unwrap();

        let result = scan_path(dir, true).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let result = scan_path(Path::new("/nonexistent/path/to/nowhere"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_results_sorted() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();

        fs::write(dir.join("z.epub"), b"test").unwrap();
        fs::write(dir.join("a.epub"), b"test").unwrap();
        fs::write(dir.join("m.epub"), b"test").unwrap();

        let result = scan_path(dir, false).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result[0].file_name().unwrap() < result[1].file_name().unwrap());
        assert!(result[1].file_name().unwrap() < result[2].file_name().unwrap());
    }
}
