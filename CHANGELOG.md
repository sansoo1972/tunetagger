# Changelog

All notable changes to TuneTagger will be documented in this file.

The format is inspired by Keep a Changelog, and the project uses semantic versioning while it evolves.

## [0.1.4] - 2026-07-23

### Added

- Added a detailed plain-text batch report with successful, skipped, and failed files.
- Added configurable report paths through `--report`.
- Added concise console failure diagnostics and the generated report location.
- Added destination-file detection before recognition and metadata processing.
- Added `--existing ask`, `--existing skip`, and `--existing process` policies.
- Added per-file interactive choices to skip, skip all remaining matches, or process a match.
- Added a guided batch setup when running `tunetagger` without a subcommand or with `tunetagger interactive`.
- Added guided prompts for source, destination, recursion, run mode, existing-file behavior, and report location.
- Added a final settings review and confirmation before guided processing begins.

### Changed

- Improved batch report readability with a compact results summary, failures first, relative paths, and numbered file lists.
- Changed batch reporting to classify recognition failures as no match, network, audio decoding, fingerprinting, validation, or invalid service response.
- Empty recognition results are now reported as `recognition / no match` instead of misleading network failures.
- Recursive batch scans automatically exclude a destination directory nested inside the source tree.
- Batch mode remains non-recursive by default.
- Guided runs default to dry-run mode and require confirmation before processing.

### Fixed

- Fixed output files being rediscovered when the destination directory was nested inside a recursively scanned source.
- Fixed repeated skip reasons and long absolute paths making large reports difficult to review.
- Fixed SongRec empty-match responses being surfaced as network errors.
- Fixed files with no metadata candidates being counted as successful batch items.
- Fixed interactive runs potentially waiting for prompts when standard input is non-interactive.

### Validated

- Confirmed custom and default batch report paths.
- Confirmed skip, process, skip-all, and non-interactive existing-file behavior.
- Confirmed nested destination directories are excluded from recursive scans.
- Confirmed the guided setup completes an end-to-end dry run and writes its report.
- Added automated coverage for report rendering, failure classification, existing-file decisions, and guided setup.

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
