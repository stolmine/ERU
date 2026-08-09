pub mod cli;
pub mod enrich;
pub mod error;
pub mod matcher;
pub mod metadata;
pub mod provider;
pub mod rename;
pub mod scanner;
pub mod writer;

pub use error::{EruError, Result};
pub use metadata::{EpubMetadata, extract_metadata};
pub use scanner::{scan_path, is_epub, is_supported, SUPPORTED_EXTS};
pub use rename::{RenameAction, RenameOptions, generate_filename, create_rename_action, execute_rename};
pub use cli::{Args, Command};
pub use provider::BookRecord;
pub use writer::WriteConfig;
pub use enrich::{enrich_file, EnrichConfig, EnrichOutcome, EnrichStatus};
