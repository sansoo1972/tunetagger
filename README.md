# TuneTagger

TuneTagger is a cross-platform Rust CLI tool for identifying MP3 files from audio, enriching them with high-quality metadata, embedding artwork and optional lyrics, and writing clean ID3 tags.

The initial workflow is intentionally conservative:

```text
MP3 input
  -> scan
  -> recognize using SongRec
  -> look up metadata
  -> score candidate matches
  -> dry-run tag diff
  -> write ID3 tags to an output copy
  -> save a sidecar audit JSON
```

Apple Music/iTunes Match is treated as a downstream use case, not a core dependency.

## Early CLI shape

```bash
tunetagger scan ./input
tunetagger recognize ./input/song.mp3
tunetagger lookup ./input/song.mp3
tunetagger tag ./input/song.mp3 --dry-run
tunetagger tag ./input/song.mp3 --write --output ./tagged
tunetagger batch ./input --output ./tagged --recursive --write
```

## Project layout

```text
crates/tunetagger-core         Shared models, config, pipeline types
crates/tunetagger-cli          CLI entry point and commands
crates/tunetagger-recognition  SongRec adapter
crates/tunetagger-metadata     Apple/iTunes metadata lookup and scoring
crates/tunetagger-tags         ID3 tag reading/writing, artwork, lyrics
crates/tunetagger-files        Scanning, backups, output naming, sidecars
```

## Status

Foundation only. The first runnable command is `scan`.
