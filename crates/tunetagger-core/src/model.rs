use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackIdentity {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration_ms: Option<u64>,
    pub isrc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataCandidate {
    pub source: MetadataSource,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u16>,
    pub track_total: Option<u16>,
    pub disc_number: Option<u16>,
    pub disc_total: Option<u16>,
    pub release_date: Option<String>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub explicit: Option<bool>,
    pub artwork_url: Option<String>,
    pub duration_ms: Option<u64>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataSource {
    SongRec,
    Apple,
    MusicBrainz,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub track_number: Option<u16>,
    pub disc_number: Option<u16>,
    pub has_artwork: bool,
    pub has_lyrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTags {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u16>,
    pub track_total: Option<u16>,
    pub disc_number: Option<u16>,
    pub disc_total: Option<u16>,
    pub release_date: Option<String>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub sort_artist: Option<String>,
    pub sort_album: Option<String>,
    pub sort_album_artist: Option<String>,
    pub lyrics: Option<Lyrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    pub language: String,
    pub description: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingAudit {
    pub source_path: std::path::PathBuf,
    pub output_path: Option<std::path::PathBuf>,
    pub identity: Option<TrackIdentity>,
    pub selected_candidate: Option<MetadataCandidate>,
    pub proposed_tags: Option<ResolvedTags>,
    pub status: ProcessingStatus,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Recognized,
    Matched,
    Tagged,
    NeedsReview,
    Failed,
}
