# Changelog

All notable changes to TuneTagger will be documented in this file.

The format is inspired by Keep a Changelog, and the project uses semantic versioning while it evolves.

## [0.1.3] - 2026-04-30

### Added

- Added batch MP3 tagging for folder processing.
- Added non-recursive batch behavior by default.
- Added optional recursive batch processing with `--recursive`.
- Added MusicBrainz credit enrichment.
- Added Composer enrichment from MusicBrainz work/artist relationships when available.
- Added safer Album Artist resolution that does not blindly force Track Artist into Album Artist.
- Added existing tag preservation for Album Artist and Composer where appropriate.
- Added batch summary output with successful and failed counts.
- Added more resilient MusicBrainz response decoding.
- Added clearer MusicBrainz enrichment warning messages.

### Changed

- Improved Album Artist resolution priority:
  - Apple/iTunes `collectionArtistName`, when present
  - MusicBrainz release artist-credit, when available
  - Existing Album Artist tag, when present
  - Conservative single-artist fallback only when low-risk
- Composer is only written when found from a trusted enrichment source or preserved from an existing tag.
- Batch processing reuses the validated single-file tagging workflow.
- Batch write outputs tagged copies to the specified output folder while preserving original filenames.

### Validated

- Confirmed actual batch write completed successfully:
  - Successful: 36
  - Failed: 0
- Confirmed batch mode is non-recursive unless `--recursive` is supplied.
- Confirmed Composer enrichment works for tracks where MusicBrainz has credit data.
- Confirmed tracks without MusicBrainz composer data are left blank rather than guessed.
- Confirmed Album Artist and Composer tags are visible through `ffprobe` when present.
- Confirmed album artwork remains embedded as an attached front-cover image.

### Known Limitations

- Output files are not renamed yet.
- Batch mode currently supports MP3 files only.
- Composer enrichment depends on MusicBrainz having recording/work relationship data.
- Some large or well-known artists may still lack Composer data if MusicBrainz has not added work relationships for the selected recording.

### Backlog

- Add output filename renaming with `--rename`.
- Add configurable output filename templates.
- Add nested output filename templates such as `{album_artist}/{album}/{track:02} - {title}.mp3`.
- Add collision-safe duplicate handling for renamed files.
- Add sidecar audit JSON.
- Add optional macOS Apple Music library update support for applying corrected TuneTagger metadata to existing Apple Music tracks.

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
