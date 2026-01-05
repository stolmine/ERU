use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "eru",
    version,
    about = "EPUB Rename Utility - Extract metadata and rename EPUB files"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List metadata for EPUB file(s)
    List {
        /// Path to EPUB file or directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Disable recursive directory scanning
        #[arg(long)]
        no_recursive: bool,
    },
    /// Rename EPUB files using a pattern
    Rename {
        /// Path to EPUB file or directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Filename pattern using placeholders like {title}, {author}
        #[arg(short, long)]
        pattern: String,

        /// Execute the rename (default is dry-run)
        #[arg(short, long)]
        execute: bool,

        /// Disable recursive directory scanning
        #[arg(long)]
        no_recursive: bool,

        /// Replace spaces in filename with this character
        #[arg(short, long)]
        space: Option<char>,

        /// Remove commas from author field (Smith Jane instead of Smith, Jane)
        #[arg(long)]
        no_comma: bool,
    },
    /// Show available pattern placeholders
    Patterns,
}
