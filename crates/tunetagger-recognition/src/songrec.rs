use std::path::Path;

use songrec::{Config, SongRec};
use tunetagger_core::{TrackIdentity, TuneTaggerError, TuneTaggerResult};

#[derive(Debug, Clone)]
pub struct SongRecRecognizer;

impl Default for SongRecRecognizer {
    fn default() -> Self {
        Self
    }
}

impl SongRecRecognizer {
    pub fn recognize_file(&self, path: impl AsRef<Path>) -> TuneTaggerResult<TrackIdentity> {
        let path = path.as_ref();

        let path_string = path.to_str().ok_or_else(|| {
            TuneTaggerError::Recognition(format!("invalid path: {}", path.display()))
        })?;

        let config = Config::default();
        let recognizer = SongRec::new(config);

        let result = recognizer
            .recognize_from_file(path_string)
            .map_err(|err| TuneTaggerError::Recognition(err.to_string()))?;

        Ok(TrackIdentity {
            title: result.song_name,
            artist: result.artist_name,
            album: result.album_name,
            album_artist: None,
            duration_ms: None,
            isrc: None,
        })
    }
}
