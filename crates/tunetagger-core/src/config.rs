use crate::errors::TuneTaggerResult;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub recognition: RecognitionConfig,
    pub metadata: MetadataConfig,
    pub scoring: ScoringConfig,
    pub tagging: TaggingConfig,
    pub safety: SafetyConfig,
    pub naming: NamingConfig,
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> TuneTaggerResult<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub input_dir: String,
    pub output_dir: String,
    pub review_dir: String,
    pub archive_dir: String,
    pub report_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionConfig {
    pub songrec_path: String,
    pub ffmpeg_path: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub primary_source: String,
    pub use_musicbrainz: bool,
    pub download_artwork: bool,
    pub artwork_size: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub auto_tag_threshold: u8,
    pub review_threshold: u8,
    pub duration_tolerance_seconds: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggingConfig {
    pub id3_version: String,
    pub embed_artwork: bool,
    pub embed_lyrics: bool,
    pub write_sort_fields: bool,
    pub preserve_existing_when_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub dry_run_default: bool,
    pub backup_before_write: bool,
    pub modify_originals: bool,
    pub copy_to_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    pub rename_output_files: bool,
    pub template: String,
    pub sanitize_filenames: bool,
}
