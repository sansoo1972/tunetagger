# TuneTagger

TuneTagger is a cross-platform Rust CLI tool for identifying MP3 files from audio, enriching them with high-quality metadata, embedding album artwork and optional lyrics, and writing clean ID3 tags.

The project started from ideas explored in `songart`, but TuneTagger is focused on batch music file cleanup rather than real-time display or Raspberry Pi playback.

Apple Music/iTunes Match is treated as a downstream use case, not a core dependency. The output remains a clean, portable MP3 file with standard ID3 metadata.

## Current milestone

Current milestone: `v0.1.3`.

Validated so far:

```text
scan         works
recognize    works
lookup       works
tag dry-run  works
tag write    works
artwork      embeds successfully
batch write  works, non-recursive by default
```

The latest validated batch run completed successfully:

```text
Batch complete.
  Successful: 36
  Failed:     0
```

## Current workflow

```text
MP3 input
  -> scan files
  -> recognize audio using SongRec library
  -> look up metadata using Apple/iTunes Search API
  -> score candidate matches
  -> enrich credits with MusicBrainz when available
  -> preview proposed tag changes
  -> write ID3 tags to an output copy
  -> embed album artwork
```

TuneTagger is intentionally conservative. It does not invent missing metadata. For example, Composer is only written when a trusted source such as MusicBrainz provides usable composer/writer data, or when an existing Composer tag is preserved.

## Current CLI

```bash
tunetagger scan ./input
tunetagger recognize ./input/song.mp3
tunetagger lookup ./input/song.mp3
tunetagger tag ./input/song.mp3 --dry-run
tunetagger tag ./input/song.mp3 --write --output ./tagged
tunetagger batch ./input --write --output ./tagged
tunetagger batch ./input --write --recursive --output ./tagged
```

During development, run through Cargo:

```bash
cargo run -p tunetagger -- scan ./input
cargo run -p tunetagger -- recognize ./input/song.mp3
cargo run -p tunetagger -- lookup ./input/song.mp3
cargo run -p tunetagger -- tag ./input/song.mp3 --dry-run
cargo run -p tunetagger -- tag ./input/song.mp3 --write --output ./tagged
cargo run -p tunetagger -- batch ./input --write --output ./tagged
```

## Examples

### Single-file dry run

```bash
cargo run -p tunetagger -- tag "/path/to/song.mp3" --dry-run
```

### Single-file write to output copy

```bash
cargo run -p tunetagger -- tag "/path/to/No Controles.mp3" \
  --write \
  --output "/path/to/tagged"
```

Example output:

```text
Best candidate score: 90.0
Proposed tags:
  Title:        None -> No Controles
  Artist:       None -> Flans
  Album:        None -> Some("Flans")
  Album Artist: None -> Some("Flans")
  Composer:     None -> None
  Track:        None -> Some(7)
  Genre:        None -> Some("Pop Latino")
Downloading artwork: https://...
Embedded artwork.
Wrote tags to /path/to/tagged/No Controles.mp3
```

### Batch write

```bash
cargo run -p tunetagger -- batch "/path/to/mp3-folder" \
  --write \
  --output "/path/to/tagged"
```

Batch mode currently processes MP3 files only. It is non-recursive unless `--recursive` is supplied.

## Project layout

```text
crates/tunetagger-core         Shared models, config, errors, and common types
crates/tunetagger-cli          CLI entry point and commands
crates/tunetagger-recognition  SongRec library adapter
crates/tunetagger-metadata     Apple/iTunes lookup, MusicBrainz enrichment, scoring
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
- Batch-tag MP3 files in a directory
- Keep batch processing non-recursive by default
- Use MusicBrainz for Composer enrichment when available
- Use MusicBrainz release artist-credit for Album Artist when available
- Preserve existing Album Artist and Composer when appropriate
- Avoid blindly forcing Track Artist into Album Artist
```

Album Artist resolution priority:

```text
1. Apple/iTunes collectionArtistName, when present
2. MusicBrainz release artist-credit, when available
3. Existing Album Artist tag, when present
4. Conservative single-artist fallback only when low-risk
```

Composer resolution priority:

```text
1. MusicBrainz work/artist relationships, when available
2. Existing Composer tag, when present
3. Blank / unresolved
```

## Safety model

TuneTagger is intentionally conservative.

The recommended workflow is to write tagged files to an output directory instead of modifying originals in place:

```bash
cargo run -p tunetagger -- tag ./input/song.mp3 --write --output ./tagged
```

For batch processing:

```bash
cargo run -p tunetagger -- batch ./input --write --output ./tagged
```

This leaves the source MP3s untouched and creates tagged copies.

Do not place the output directory inside the input tree when using `--recursive`, or already-tagged output files may be discovered on future recursive runs.

## Requirements

- Rust 1.78 or newer
- Network access for metadata, MusicBrainz enrichment, and artwork lookup
- MP3 input files

TuneTagger currently uses the `songrec-lib` crate directly, so a separate `songrec` CLI binary is not required.

## Roadmap / Backlog

Planned:

```text
- Sidecar audit JSON
- Better candidate review flow
- Optional lyrics embedding
- Configurable metadata sources
- MusicBrainz lookup improvements
- Safer overwrite/backup policies
- Output filename renaming with --rename
- Configurable filename templates such as {artist} - {title}.mp3
- Nested filename templates such as {album_artist}/{album}/{track:02} - {title}.mp3
- Safe filename sanitization and duplicate collision handling
- Support for additional audio formats
- Optional macOS Apple Music library update feature for applying corrected TuneTagger metadata to existing Apple Music tracks
```

The Apple Music feature is a future macOS-only add-on. The core TuneTagger workflow remains cross-platform MP3 tagging.

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

## Notes

TuneTagger currently keeps output filenames unchanged. Renaming tagged output files is planned for a future branch.

Current output behavior:

```text
Input:
  Aliens - YouTube.mp3

Output:
  tagged/Aliens - YouTube.mp3
```

Future output-renaming behavior may support:

```text
tagged/BTS - Aliens.mp3
tagged/Alphaville - A Victory of Love.mp3
tagged/Dominic Fike - Babydoll.mp3
```
