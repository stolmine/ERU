use clap::Parser;
use eru::{Args, Command, RenameOptions, create_rename_action, execute_rename, extract_metadata, scan_path};
use std::path::Path;
use std::process;

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Command::List { path, no_recursive } => handle_list(&path, !no_recursive),
        Command::Rename { path, pattern, execute, no_recursive, space, no_comma } => {
            let opts = RenameOptions { space_char: space, no_comma };
            handle_rename(&path, &pattern, execute, !no_recursive, &opts)
        }
        Command::Patterns => {
            handle_patterns();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn handle_list(path: &Path, recursive: bool) -> eru::Result<()> {
    let epubs = scan_path(path, recursive)?;

    if epubs.is_empty() {
        println!("No EPUB files found.");
        return Ok(());
    }

    for epub_path in epubs {
        match extract_metadata(&epub_path) {
            Ok(metadata) => print_metadata(&metadata),
            Err(e) => eprintln!("Error reading {}: {}", epub_path.display(), e),
        }
    }

    Ok(())
}

fn print_metadata(metadata: &eru::EpubMetadata) {
    println!("Path: {}", metadata.source_path.display());
    println!("  Title:     {}", metadata.title.as_deref().unwrap_or("N/A"));
    println!("  Author:    {}", metadata.author.as_deref().unwrap_or("N/A"));
    println!("  Publisher: {}", metadata.publisher.as_deref().unwrap_or("N/A"));
    println!("  Date:      {}", metadata.date.as_deref().unwrap_or("N/A"));
    println!("  ISBN:      {}", metadata.isbn.as_deref().unwrap_or("N/A"));
    println!("  Language:  {}", metadata.language.as_deref().unwrap_or("N/A"));
    println!();
}

fn handle_rename(path: &Path, pattern: &str, execute: bool, recursive: bool, opts: &RenameOptions) -> eru::Result<()> {
    let epubs = scan_path(path, recursive)?;

    if epubs.is_empty() {
        println!("No EPUB files found.");
        return Ok(());
    }

    let mut skip_all = false;
    let mut yes_all = false;

    for epub_path in epubs {
        let metadata = match extract_metadata(&epub_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error reading {}: {}", epub_path.display(), e);
                continue;
            }
        };

        if !metadata.has_metadata() {
            eprintln!("Skipping {}: no metadata", epub_path.display());
            continue;
        }

        let action = create_rename_action(&metadata, pattern, opts);

        if action.from == action.to {
            continue;
        }

        if !execute {
            println!("{} -> {}", action.from.display(), action.to.display());
            continue;
        }

        if action.to.exists() {
            if skip_all {
                continue;
            }

            if !yes_all {
                match prompt_conflict(&action) {
                    ConflictChoice::Yes => {}
                    ConflictChoice::No => continue,
                    ConflictChoice::All => yes_all = true,
                    ConflictChoice::SkipAll => {
                        skip_all = true;
                        continue;
                    }
                }
            }
        }

        match execute_rename(&action) {
            Ok(_) => println!("Renamed: {} -> {}", action.from.display(), action.to.display()),
            Err(e) => eprintln!("Failed to rename {}: {}", action.from.display(), e),
        }
    }

    Ok(())
}

enum ConflictChoice {
    Yes,
    No,
    All,
    SkipAll,
}

fn prompt_conflict(action: &eru::RenameAction) -> ConflictChoice {
    eprintln!("\nFile already exists: {}", action.to.display());
    eprintln!("Overwrite?");
    eprintln!("  [y] Yes");
    eprintln!("  [n] No");
    eprintln!("  [a] All");
    eprintln!("  [s] Skip all");

    loop {
        let input = dialoguer::Input::<String>::new()
            .with_prompt("Choice")
            .interact_text()
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        match input.as_str() {
            "y" | "yes" => return ConflictChoice::Yes,
            "n" | "no" => return ConflictChoice::No,
            "a" | "all" => return ConflictChoice::All,
            "s" | "skip" | "skip all" => return ConflictChoice::SkipAll,
            _ => eprintln!("Invalid choice. Please enter y, n, a, or s."),
        }
    }
}

fn handle_patterns() {
    println!("Available pattern placeholders:");
    println!("  {{title}}     - Book title");
    println!("  {{author}}    - Book author");
    println!("  {{publisher}} - Publisher name");
    println!("  {{date}}      - Publication date");
    println!("  {{year}}      - Publication year (extracted from date)");
    println!("  {{isbn}}      - ISBN identifier");
    println!("  {{language}}  - Language code");
    println!("\nExample: eru rename book.epub -p \"{{author}} - {{title}} ({{year}})\"");
}
