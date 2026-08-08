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
    /// Fetch canonical metadata online, write it into the EPUB, and rename (dry-run by default)
    Enrich {
        /// Path to EPUB file or directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Filename pattern using placeholders like {title}, {author}
        #[arg(short, long, default_value = "{author} - {title} ({year})")]
        pattern: String,

        /// Execute the enrichment: write metadata + rename (default is dry-run)
        #[arg(short, long)]
        execute: bool,

        /// Minimum match confidence [0.0-1.0] to auto-apply; below this a file is flagged
        #[arg(long, default_value_t = 0.75)]
        min_confidence: f32,

        /// Move enriched files into this directory (e.g. the CWA ingest folder)
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// ebook-meta invocation, space-separated
        /// (e.g. "docker exec -i calibre-web-automated ebook-meta")
        #[arg(long, default_value = "ebook-meta")]
        ebook_meta_cmd: String,

        /// host:container path-prefix map when ebook-meta runs in a container
        /// (e.g. "/mnt/storage/downloads/book-ingest:/cwa-book-ingest")
        #[arg(long)]
        path_map: Option<String>,

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
