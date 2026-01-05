use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum EruError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("EPUB parsing error: {0}")]
    EpubParse(#[from] epub::doc::DocError),

    #[error("Invalid rename pattern: {0}")]
    InvalidPattern(String),

    #[error("Metadata extraction failed: {0}")]
    MetadataExtraction(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Not an EPUB file: {0}")]
    NotAnEpub(PathBuf),
}

pub type Result<T> = std::result::Result<T, EruError>;
