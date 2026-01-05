pub mod cli;
pub mod error;
pub mod metadata;
pub mod rename;
pub mod scanner;

pub use error::{EruError, Result};
pub use metadata::{EpubMetadata, extract_metadata};
pub use scanner::scan_path;
pub use rename::{RenameAction, RenameOptions, generate_filename, create_rename_action, execute_rename};
pub use cli::{Args, Command};
