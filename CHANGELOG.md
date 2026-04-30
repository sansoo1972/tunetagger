# Changelog

All notable changes to TuneTagger will be documented in this file.

The format is inspired by Keep a Changelog, and the project uses semantic versioning while it evolves.

## [0.1.2] - 2026-04-29

### Added

- Added improved metadata candidate scoring.
- Added album artwork download support.
- Added embedded front-cover artwork writing to MP3 ID3 tags.
- Added artwork handling through the `tunetagger-tags` crate.
- Added safe output-copy tagging workflow using `--output`.

### Changed

- Tuned scoring so exact title, artist, and album matches score higher.
- A clean title + artist + album match now lands around `90.0` instead of `70.0`.
- Updated tag write flow to embed artwork after writing text metadata.

### Validated

- Confirmed recognition works against `No Controles.mp3`.
- Confirmed lookup returns Apple/iTunes candidates.
- Confirmed dry-run tag preview works.
- Confirmed writing tags to an output copy works.
- Confirmed album artwork embeds successfully.

## [0.1.1] - 2026-04-29

### Added

- Added direct SongRec recognition using `songrec-lib`.
- Removed need for an external `songrec` CLI binary.
- Added recognition flow from MP3 audio to normalized `TrackIdentity`.
- Added Apple/iTunes metadata lookup.
- Added lookup scoring flow.
- Added `recognize` command.
- Added `lookup` command.
- Added initial `tag --dry-run` behavior.

### Changed

- Reworked recognition adapter from shelling out to `songrec` to using the SongRec library directly.
- Moved blocking recognition work into `tokio::task::spawn_blocking` to avoid Tokio runtime shutdown panics.

### Fixed

- Fixed initial compile issues in the generated Rust workspace.
- Fixed missing dependency declarations.
- Fixed ID3 existing tag model mismatches.
- Fixed path handling for SongRec recognition.

### Validated

- Confirmed `scan` command detects MP3 files.
- Confirmed `recognize` identifies:
  - Title: `No Controles`
  - Artist: `Flans`
  - Album: `Flans`

## [0.1.0] - 2026-04-29

### Added

- Created initial TuneTagger Rust workspace.
- Added core project structure:
  - `tunetagger-core`
  - `tunetagger-cli`
  - `tunetagger-recognition`
  - `tunetagger-metadata`
  - `tunetagger-tags`
  - `tunetagger-files`
- Added initial CLI command skeleton.
- Added config file structure.
- Added shared metadata, recognition, and tagging models.
- Added ID3 reader/writer skeleton.
- Added file scanning support.
- Added initial project documentation.

### Status

- Foundation release.
- First runnable command: `scan`.
