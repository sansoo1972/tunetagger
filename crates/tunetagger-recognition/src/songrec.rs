use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use songrec::{Config, SongRec, SongRecError};
use tunetagger_core::{TrackIdentity, TuneTaggerError, TuneTaggerResult};

const SAMPLE_RATE: usize = 16_000;
const WINDOW_SECONDS: usize = 12;
const WINDOW_SAMPLES: usize = SAMPLE_RATE * WINDOW_SECONDS;

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
        let mut decoded_samples = None;
        let mut attempted_starts = HashSet::new();

        run_recognition_attempts(|window| {
            let result = match window {
                RecognitionWindow::Middle => recognizer.recognize_from_file(path_string),
                RecognitionWindow::Quarter | RecognitionWindow::ThreeQuarter => {
                    let samples = match decoded_samples.as_ref() {
                        Some(samples) => samples,
                        None => {
                            decoded_samples = Some(decode_audio(path)?);
                            let samples =
                                decoded_samples.as_ref().expect("samples were just decoded");
                            let (middle_start, _) =
                                select_window(samples, RecognitionWindow::Middle);
                            attempted_starts.insert(middle_start);
                            samples
                        }
                    };
                    let (start, window_samples) = select_window(samples, window);
                    if !attempted_starts.insert(start) {
                        return Ok(RecognitionAttempt::SkippedDuplicate);
                    }
                    recognizer.recognize_from_samples(window_samples, SAMPLE_RATE as u32)
                }
            };

            match result {
                Ok(result) => Ok(RecognitionAttempt::Matched(track_identity(result))),
                Err(error) if is_no_match(&error) => Ok(RecognitionAttempt::NoMatch),
                Err(error) => Err(map_songrec_error(error)),
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecognitionWindow {
    Middle,
    Quarter,
    ThreeQuarter,
}

impl RecognitionWindow {
    const ALL: [Self; 3] = [Self::Middle, Self::Quarter, Self::ThreeQuarter];

    fn fraction(self) -> f64 {
        match self {
            Self::Middle => 0.5,
            Self::Quarter => 0.25,
            Self::ThreeQuarter => 0.75,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Middle => "50%",
            Self::Quarter => "25%",
            Self::ThreeQuarter => "75%",
        }
    }
}

enum RecognitionAttempt<T> {
    Matched(T),
    NoMatch,
    SkippedDuplicate,
}

fn run_recognition_attempts<T, F>(mut attempt: F) -> TuneTaggerResult<T>
where
    F: FnMut(RecognitionWindow) -> TuneTaggerResult<RecognitionAttempt<T>>,
{
    let mut attempted = Vec::new();

    for window in RecognitionWindow::ALL {
        match attempt(window)? {
            RecognitionAttempt::Matched(result) => return Ok(result),
            RecognitionAttempt::NoMatch => attempted.push(window.label()),
            RecognitionAttempt::SkippedDuplicate => {}
        }
    }

    Err(TuneTaggerError::RecognitionNoMatch(format!(
        "the recognition service returned no matching track after {} distinct 12-second \
         attempt(s) at {} of the track",
        attempted.len(),
        attempted.join(", ")
    )))
}

fn decode_audio(path: &Path) -> TuneTaggerResult<Vec<i16>> {
    let file = File::open(path).map_err(|error| {
        TuneTaggerError::RecognitionAudio(format!("failed to open '{}': {error}", path.display()))
    })?;
    let decoder = rodio::Decoder::new(BufReader::new(file)).map_err(|error| {
        TuneTaggerError::RecognitionAudio(format!("failed to decode '{}': {error}", path.display()))
    })?;
    let samples = rodio::source::UniformSourceIterator::new(decoder, 1, SAMPLE_RATE as u32)
        .collect::<Vec<_>>();

    if samples.len() < SAMPLE_RATE * 3 {
        return Err(TuneTaggerError::RecognitionAudio(format!(
            "'{}' contains {:.2} seconds of decodable audio; at least 3 seconds are required",
            path.display(),
            samples.len() as f64 / SAMPLE_RATE as f64
        )));
    }

    Ok(samples)
}

fn select_window(samples: &[i16], window: RecognitionWindow) -> (usize, &[i16]) {
    let length = samples.len().min(WINDOW_SAMPLES);
    let maximum_start = samples.len().saturating_sub(length);
    let center = (samples.len() as f64 * window.fraction()) as usize;
    let start = center.saturating_sub(length / 2).min(maximum_start);
    let usable_length = length - (length % 128);
    (start, &samples[start..start + usable_length])
}

fn track_identity(result: songrec::RecognitionResult) -> TrackIdentity {
    TrackIdentity {
        title: result.song_name,
        artist: result.artist_name,
        album: result.album_name,
        album_artist: None,
        duration_ms: None,
        isrc: None,
    }
}

fn is_no_match(error: &SongRecError) -> bool {
    matches!(
        error,
        SongRecError::NetworkError(message)
            if message.eq_ignore_ascii_case("No track found in response")
    )
}

fn map_songrec_error(error: SongRecError) -> TuneTaggerError {
    match error {
        error if is_no_match(&error) => TuneTaggerError::RecognitionNoMatch(
            "the recognition service returned no matching track".to_owned(),
        ),
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
    fn succeeds_without_fallback_when_middle_matches() {
        let mut windows = Vec::new();
        let result = run_recognition_attempts(|window| {
            windows.push(window);
            Ok(RecognitionAttempt::Matched("matched"))
        })
        .unwrap();

        assert_eq!(result, "matched");
        assert_eq!(windows, vec![RecognitionWindow::Middle]);
    }

    #[test]
    fn succeeds_on_a_fallback_window() {
        let mut windows = Vec::new();
        let result = run_recognition_attempts(|window| {
            windows.push(window);
            if window == RecognitionWindow::ThreeQuarter {
                Ok(RecognitionAttempt::Matched("matched"))
            } else {
                Ok(RecognitionAttempt::NoMatch)
            }
        })
        .unwrap();

        assert_eq!(result, "matched");
        assert_eq!(windows, RecognitionWindow::ALL);
    }

    #[test]
    fn exhausted_attempts_report_distinct_windows() {
        let error = run_recognition_attempts::<(), _>(|window| {
            if window == RecognitionWindow::Quarter {
                Ok(RecognitionAttempt::SkippedDuplicate)
            } else {
                Ok(RecognitionAttempt::NoMatch)
            }
        })
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("2 distinct 12-second attempt(s)"));
        assert!(error.to_string().contains("50%, 75%"));
    }

    #[test]
    fn non_retryable_failures_stop_immediately() {
        let mut attempts = 0;
        let error = run_recognition_attempts::<(), _>(|_| {
            attempts += 1;
            Err(TuneTaggerError::Network("connection timed out".to_owned()))
        })
        .unwrap_err();

        assert_eq!(attempts, 1);
        assert!(matches!(error, TuneTaggerError::Network(_)));
    }

    #[test]
    fn selects_distinct_windows_for_a_full_track() {
        let samples = vec![0; SAMPLE_RATE * 120];
        let (middle, middle_samples) = select_window(&samples, RecognitionWindow::Middle);
        let (quarter, quarter_samples) = select_window(&samples, RecognitionWindow::Quarter);
        let (three_quarter, three_quarter_samples) =
            select_window(&samples, RecognitionWindow::ThreeQuarter);

        assert_eq!(middle_samples.len(), WINDOW_SAMPLES);
        assert_eq!(quarter_samples.len(), WINDOW_SAMPLES);
        assert_eq!(three_quarter_samples.len(), WINDOW_SAMPLES);
        assert!(quarter < middle);
        assert!(middle < three_quarter);
    }

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
