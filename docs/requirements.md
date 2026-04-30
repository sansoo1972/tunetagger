# TuneTagger Requirements

## Goal

TuneTagger identifies MP3 files from their audio, enriches them with high-quality metadata, writes standards-compliant ID3 tags, embeds artwork and optional lyrics, and outputs clean MP3 files suitable for Apple Music, Plex, Jellyfin, Navidrome, iTunes Match, and general music-library use.

## In scope

- MP3 input files
- ID3 tag reading and writing
- Folder scanning
- SongRec-based recognition
- Apple/iTunes Search API metadata lookup
- Candidate scoring
- Conservative dry-run diffing
- Output-copy tagging by default
- Artwork embedding
- Optional lyrics from a local text file
- JSON sidecar audit files

## Out of scope for MVP

- Apple Music import automation
- iTunes Match verification
- AAC replacement workflow
- Playlist management
- Cloud Status inspection
- Streaming-service sync
- Fully automated lyrics scraping

## Safety defaults

- Dry-run first
- Do not modify originals unless explicitly requested
- Prefer copy-to-output tagging
- Preserve existing user-curated metadata when confidence is low
- Save sidecar audit records for traceability
