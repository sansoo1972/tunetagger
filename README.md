# TuneTagger

TuneTagger is a cross-platform Rust CLI tool for identifying MP3 files from audio, enriching them with high-quality metadata, embedding album artwork and optional lyrics, and writing clean ID3 tags.

The project started from ideas explored in `songart`, but TuneTagger is focused on batch music file cleanup rather than real-time display or Raspberry Pi playback.

Apple Music/iTunes Match is treated as a downstream use case, not a core dependency. The output remains a clean, portable MP3 file with standard ID3 metadata.

## Current workflow

```text
MP3 input
  -> scan files
  -> recognize audio using SongRec library
  -> look up metadata using Apple/iTunes Search API
  -> score candidate matches
  -> preview proposed tag changes
  -> write ID3 tags to an output copy
  -> embed album artwork
```

## Current CLI

```bash
tunetagger scan ./input
tunetagger recognize ./input/song.mp3
tunetagger lookup ./input/song.mp3
tunetagger tag ./input/song.mp3 --dry-run
tunetagger tag ./input/song.mp3 --write --output ./tagged
```

During development, run through Cargo:

```bash
cargo run -p tunetagger -- scan ./input
cargo run -p tunetagger -- recognize ./input/song.mp3
cargo run -p tunetagger -- lookup ./input/song.mp3
cargo run -p tunetagger -- tag ./input/song.mp3 --dry-run
cargo run -p tunetagger -- tag ./input/song.mp3 --write --output ./tagged
```

## Example

```bash
cargo run -p tunetagger -- tag "/path/to/No Controles.mp3" \
  --write \
  --output "/path/to/tagged"
```

Example output:

```text
Best candidate score: 90.0
Proposed tags:
  Title:  None -> No Controles
  Artist: None -> Flans
  Album:  None -> Some("Flans")
  Track:  None -> Some(7)
  Genre:  None -> Some("Pop Latino")
Downloading artwork: https://...
Embedded artwork.
Wrote tags to /path/to/tagged/No Controles.mp3
```

## Project layout

```text
crates/tunetagger-core         Shared models, config, errors, and common types
crates/tunetagger-cli          CLI entry point and commands
crates/tunetagger-recognition  SongRec library adapter
crates/tunetagger-metadata     Apple/iTunes metadata lookup and scoring
crates/tunetagger-tags         ID3 tag reading/writing, artwork, lyrics helpers
crates/tunetagger-files        Scanning, backups, output naming, sidecars
```

## Features

Current:

```text
- Scan folders for MP3 files
- Recognize MP3 files from audio using songrec-lib
- Query Apple/iTunes metadata candidates
- Score metadata candidates
- Preview tag changes
- Write ID3 tags
- Copy tagged files to an output directory
- Download and embed album artwork
```

Planned:

```text
- Batch processing
- Sidecar audit JSON
- Better candidate review flow
- Optional lyrics embedding
- Configurable metadata sources
- MusicBrainz lookup
- Safer overwrite/backup policies
- More robust filename normalization
- Support for additional audio formats
```

## Safety model

TuneTagger is intentionally conservative.

The recommended workflow is to write tagged files to an output directory instead of modifying originals in place:

```bash
cargo run -p tunetagger -- tag ./input/song.mp3 --write --output ./tagged
```

This leaves the source MP3 untouched and creates a tagged copy.

## Requirements

- Rust 1.78 or newer
- Network access for metadata and artwork lookup
- MP3 input files

TuneTagger currently uses the `songrec-lib` crate directly, so a separate `songrec` CLI binary is not required.

## Development

Run formatting and checks:

```bash
cargo fmt
cargo check
```

Run tests when added:

```bash
cargo test
```

Build:

```bash
cargo build
```

Build release binary:

```bash
cargo build --release
```

## Status

Early development.

Validated so far:

```text
scan      works
recognize works
lookup    works
tag dry-run works
tag write works
artwork embed works
```

Current milestone: `v0.1.2`.
