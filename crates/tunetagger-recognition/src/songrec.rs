use std::path::Path;

use songrec::{Config, SongRec, SongRecError};
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
            .map_err(map_songrec_error)?;

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

fn map_songrec_error(error: SongRecError) -> TuneTaggerError {
    match error {
        SongRecError::NetworkError(message)
            if message.eq_ignore_ascii_case("No track found in response") =>
        {
            TuneTaggerError::RecognitionNoMatch(
                "the recognition service returned no matching track".to_owned(),
            )
        }
        SongRecError::NetworkError(message)
            if message
                .to_ascii_lowercase()
                .contains("invalid response format") =>
        {
            TuneTaggerError::RecognitionService(message)
        }
        SongRecError::NetworkError(message) => TuneTaggerError::Network(message),
        SongRecError::AudioError(message) => TuneTaggerError::RecognitionAudio(message),
        SongRecError::FingerprintingError(message) => {
            TuneTaggerError::RecognitionFingerprint(message)
        }
        SongRecError::InvalidInput(message) => TuneTaggerError::Validation(message),
        SongRecError::ConfigError(message) => TuneTaggerError::RecognitionService(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_empty_match_response_to_no_match() {
        let error = map_songrec_error(SongRecError::NetworkError(
            "No track found in response".to_owned(),
        ));

        assert!(matches!(error, TuneTaggerError::RecognitionNoMatch(_)));
        assert!(!error.to_string().contains("Network error"));
    }

    #[test]
    fn preserves_actual_network_failures() {
        let error = map_songrec_error(SongRecError::NetworkError(
            "connection timed out".to_owned(),
        ));

        assert!(matches!(error, TuneTaggerError::Network(_)));
        assert!(error.to_string().contains("connection timed out"));
    }

    #[test]
    fn distinguishes_audio_and_fingerprinting_failures() {
        let audio = map_songrec_error(SongRecError::AudioError("bad stream".to_owned()));
        let fingerprint =
            map_songrec_error(SongRecError::FingerprintingError("too short".to_owned()));

        assert!(matches!(audio, TuneTaggerError::RecognitionAudio(_)));
        assert!(matches!(
            fingerprint,
            TuneTaggerError::RecognitionFingerprint(_)
        ));
    }
}
