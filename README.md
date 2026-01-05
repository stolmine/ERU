# ERU - EPUB Rename Utility

A fast CLI tool for extracting metadata from EPUB files and batch renaming them using customizable patterns.

## Installation

```bash
cargo install --path .
```

## Usage

### List Metadata

Display metadata for a single EPUB or all EPUBs in a directory:

```bash
eru list                         # current directory
eru list book.epub               # single file
eru list ./ebooks/               # specific directory
eru list ./ebooks/ --no-recursive
```

Output:
```
Path: ./ebooks/978-0-123456-78-9.epub
  Title:     The Great Novel
  Author:    Smith, Jane
  Publisher: Acme Publishing
  Date:      2023-05-15
  ISBN:      978-0-123456-78-9
  Language:  en
```

### Rename Files

Rename EPUBs using metadata patterns. Dry-run by default:

```bash
# Preview renames in current directory (dry-run)
eru rename -p "{author} - {title}"

# Execute renames
eru rename -p "{author} - {title}" --execute

# Specific directory
eru rename ./ebooks/ -p "{author} - {title}" --execute

# Single file
eru rename book.epub -p "{author} - {title} ({year})" --execute
```

### Pattern Placeholders

```bash
eru patterns
```

| Placeholder   | Description                              |
|---------------|------------------------------------------|
| `{title}`     | Book title                               |
| `{author}`    | Book author (Lastname, Firstname format) |
| `{publisher}` | Publisher name                           |
| `{date}`      | Publication date                         |
| `{year}`      | Year (extracted from date)               |
| `{isbn}`      | ISBN identifier                          |
| `{language}`  | Language code                            |

### Examples

```bash
# Author - Title format
eru rename ./books/ -p "{author} - {title}" --execute
# Result: Smith, Jane - The Great Novel.epub

# Include year
eru rename ./books/ -p "{author} - {title} ({year})" --execute
# Result: Smith, Jane - The Great Novel (2023).epub

# Replace spaces with underscores
eru rename ./books/ -p "{author} - {title}" -s _ --execute
# Result: Smith,_Jane_-_The_Great_Novel.epub

# Replace spaces with dots
eru rename ./books/ -p "{author} - {title}" -s . --execute
# Result: Smith,.Jane.-.The.Great.Novel.epub

# Remove comma from author name
eru rename ./books/ -p "{author} - {title}" --no-comma --execute
# Result: Smith Jane - The Great Novel.epub

# Combine options
eru rename ./books/ -p "{author} - {title}" -s _ --no-comma --execute
# Result: Smith_Jane_-_The_Great_Novel.epub
```

## Conflict Resolution

When `--execute` encounters an existing file, you'll be prompted:

```
File already exists: Jane Smith - The Great Novel.epub
Overwrite?
  [y] Yes       - Overwrite this file
  [n] No        - Skip this file
  [a] All       - Overwrite all remaining conflicts
  [s] Skip all  - Skip all remaining conflicts
```

## Options

| Flag            | Description                                 |
|-----------------|---------------------------------------------|
| `-p, --pattern` | Filename pattern with placeholders          |
| `-e, --execute` | Actually rename files (default: dry-run)    |
| `-s, --space`   | Replace spaces with given character         |
| `--no-comma`    | Remove comma from author (Smith Jane)       |
| `--no-recursive`| Only scan immediate directory, not subdirs  |
| `-h, --help`    | Show help                                   |
| `-V, --version` | Show version                                |

## Notes

- Path defaults to current directory if not specified
- Author names are formatted as "Lastname, Firstname"
- Files with no metadata (no title or author) are skipped during rename
- Missing metadata fields are omitted from the filename (not replaced with "Unknown")
- Invalid filename characters (`/ \ : * ? " < > |`) are replaced with `_`
- Recursive scanning is enabled by default
