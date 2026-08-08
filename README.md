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

### Enrich Metadata (online lookup)

`rename` only reads what's already in the file. `enrich` goes further: it looks the book up
online (Open Library — by ISBN if present, else title/author), scores the match, and — when
confidence clears a gate — **writes the canonical metadata back into the EPUB** and renames it.
Files below the gate are left untouched and flagged, so a bad guess never silently overwrites a
book. Dry-run by default.

```bash
# Preview what would be fetched/written/renamed (dry-run)
eru enrich ./inbox/

# Apply: write metadata into each EPUB + rename
eru enrich ./inbox/ --execute

# Tune the auto-apply gate (default 0.75) and the output name
eru enrich ./inbox/ -p "{author} - {title} ({year})" --min-confidence 0.8 --execute

# Move enriched files into another folder (e.g. a Calibre-Web ingest dir)
eru enrich ./inbox/ --out /srv/cwa-ingest --execute
```

Writing uses Calibre's `ebook-meta`. It doesn't have to be on `$PATH` — point `--ebook-meta-cmd`
at any invocation, and use `--path-map host:container` when it runs inside a container:

```bash
eru enrich ./inbox/ --execute \
  --ebook-meta-cmd "docker exec calibre-web-automated /usr/bin/ebook-meta" \
  --path-map "/mnt/storage/downloads/book-ingest:/cwa-book-ingest"
```

Reported per file: `✓` applied · `•` dry-run preview · `?` below the confidence gate ·
`✗` no online match · `-` no title/author/ISBN to search on.

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
