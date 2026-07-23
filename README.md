# TuneTagger

TuneTagger is a cross-platform Rust CLI tool for identifying MP3 files from audio, enriching them with high-quality metadata, embedding album artwork and optional lyrics, and writing clean ID3 tags.

The project started from ideas explored in `songart`, but TuneTagger is focused on batch music file cleanup rather than real-time display or Raspberry Pi playback.

Apple Music/iTunes Match is treated as a downstream use case, not a core dependency. The output remains a clean, portable MP3 file with standard ID3 metadata.

## Current milestone

Current milestone: `v0.1.5`.

Validated so far:

```text
scan         works
recognize    works
lookup       works
tag dry-run  works
tag write    works
artwork      embeds successfully
batch write  works, non-recursive by default
batch report works with detailed outcomes
existing destination handling works
guided setup works
three-window recognition fallback works
```

The latest validated batch run completed successfully:

```text
Batch complete.
  Successful: 0
  Skipped:    1
  Failed:     0
  Report:     batch-report.txt
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

## Recommended workflow: guided setup

For normal use, run TuneTagger without a subcommand:

```bash
tunetagger
```

When running from this source repository, use:

```bash
cd ~/Documents/Projects/tunetagger
cargo run -p tunetagger
```

TuneTagger starts an interactive wizard and guides you through the complete
batch setup. You do not need to remember the batch arguments.

### What the wizard asks

1. **Operation** — select batch tagging or quit without making changes.
2. **Source folder** — the folder containing the MP3 files to examine. The
   folder must already exist. Paths containing spaces and paths beginning with
   `~/` are supported.
3. **Destination folder** — where tagged copies should be written. Press Enter
   to use `<source>/batch_tagged`.
4. **Include subfolders** — choose whether MP3 files inside artist, album, or
   other nested folders should be included. The safe default is `No`.
5. **Run mode**:

   - `dry run` recognizes tracks and previews proposed metadata without writing
     tagged copies.
   - `write` creates tagged copies in the destination folder. Source MP3 files
     remain unchanged.

6. **Existing destination files**:

   - `ask` prompts for each matching destination filename.
   - `skip` ignores all matching destination files for this run.
   - `process` processes matches even when the destination filename exists.

7. **Report file** — press Enter to create `batch-report.txt`, or enter another
   path.
8. **Confirmation** — review every selection and approve the run. Answering
   `No` cancels without processing files.

Example:

```text
TuneTagger guided setup
=======================
1. Batch-tag an MP3 folder
q. Quit
Choose an operation [1]:
Source folder: ~/Music/to-tag
Destination folder [/Users/you/Music/to-tag/batch_tagged]:
Include MP3 files from subfolders? [y/N]: n
Run mode: dry run or write tagged copies? [d/w, default d]: d
When a destination file exists: ask, skip, or process? [a/s/p, default a]: a
Report file [batch-report.txt]:

Review settings
---------------
Source:             /Users/you/Music/to-tag
Destination:        /Users/you/Music/to-tag/batch_tagged
Include subfolders: no
Mode:               dry run
Existing files:     ask case by case
Report:             batch-report.txt
Start with these settings? [y/N]:
```

If the destination is inside the source folder, TuneTagger excludes it from
recursive scans. This prevents previously tagged output files from being
processed again.

When `ask` is selected and a matching destination file is found, enter:

- `s` to skip only that file.
- `a` to skip that file and all later matches in the current run.
- `p` to process that file and continue asking about later matches.

At completion, the console shows successful, skipped, and failed counts plus
the report location. The report lists files by outcome and includes specific
failure categories such as `recognition / no match`, `network`,
`audio decoding`, and `fingerprinting`.

The same wizard can be started explicitly:

```bash
tunetagger interactive
```

## Advanced and automated CLI usage

Manual arguments remain fully supported for scripts, scheduled jobs, and users
who want deterministic non-interactive runs. For ordinary terminal use, the
guided setup above is the recommended interface.

```bash
tunetagger scan ./input
tunetagger recognize ./input/song.mp3
tunetagger lookup ./input/song.mp3
tunetagger tag ./input/song.mp3 --dry-run
tunetagger tag ./input/song.mp3 --write --output ./tagged
tunetagger batch ./input --write --output ./tagged
tunetagger batch ./input --write --recursive --output ./tagged
tunetagger batch ./input --write --output ./tagged --report ./reports/batch.txt
tunetagger batch ./input --write --output ./tagged --existing skip
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
After processing, it writes a plain-text report to `batch-report.txt`. Use `--report <path>`
to choose another location. The report lists every successful file and every failed file,
including the failure category and detailed reason.

Before processing a file, batch mode checks for the same filename in the output
directory. The default `--existing ask` policy prompts when a match is found:
skip it, skip all further matches for this run, or process it. Automated and
non-interactive runs should select `--existing skip` or `--existing process`.
Skipped matches are listed separately in the console summary and batch report.

Recognition failures are categorized in the report. An unmatched recording is
reported as `recognition / no match`; real connectivity failures are reported
as `network`. Audio decoding, fingerprinting, validation, and malformed service
responses are shown separately. When the initial middle-of-track fingerprint
returns no match, TuneTagger retries distinct 12-second windows near 25% and
75% of the track. If all attempts fail, the report lists the windows that were
tried.

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
- Generate readable batch reports with successful, skipped, and failed files
- Detect existing destination files before recognition and metadata requests
- Guide interactive users through batch setup and confirmation
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

The recommended workflow is to start the guided setup and perform a dry run:

```bash
cargo run -p tunetagger
```

After reviewing the dry-run results and batch report, run the wizard again and
select write mode. TuneTagger writes tagged copies to the destination rather
than modifying the source MP3 files.

The equivalent advanced CLI command is:

```bash
cargo run -p tunetagger -- tag ./input/song.mp3 --write --output ./tagged
```

For batch processing:

```bash
cargo run -p tunetagger -- batch ./input --write --output ./tagged
```

This leaves the source MP3s untouched and creates tagged copies.

An output directory may be nested inside the input tree. TuneTagger automatically
excludes the destination subtree from recursive scans so output files are not
processed again.

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
