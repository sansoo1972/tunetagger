use std::{path::Path, process::Stdio, time::Duration};
use tokio::{process::Command, time::timeout};
use tunetagger_core::{
    RecognitionResult, RecognitionSource, TrackIdentity, TuneTaggerError, TuneTaggerResult,
};

#[derive(Debug, Clone)]
pub struct SongRecRecognizer {
    songrec_path: String,
    timeout_seconds: u64,
}

impl SongRecRecognizer {
    pub fn new(songrec_path: impl Into<String>, timeout_seconds: u64) -> Self {
        Self {
            songrec_path: songrec_path.into(),
            timeout_seconds,
        }
    }

    pub async fn recognize_file(
        &self,
        path: impl AsRef<Path>,
    ) -> TuneTaggerResult<RecognitionResult> {
        // SongRec CLI output can vary by version. This foundation captures stdout
        // and leaves robust parsing for the next implementation pass.
        let mut command = Command::new(&self.songrec_path);
        command
            .arg("recognize")
            .arg(path.as_ref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = timeout(Duration::from_secs(self.timeout_seconds), command.output())
            .await
            .map_err(|_| TuneTaggerError::Recognition("SongRec timed out".to_string()))??;

        if !output.status.success() {
            return Err(TuneTaggerError::Recognition(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        parse_songrec_stdout(&stdout)
    }
}

fn parse_songrec_stdout(stdout: &str) -> TuneTaggerResult<RecognitionResult> {
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        return Err(TuneTaggerError::Recognition(
            "SongRec returned empty output".to_string(),
        ));
    }

    // Placeholder parser. Next pass should parse JSON if available or use a
    // version-specific text parser.
    Ok(RecognitionResult {
        source: RecognitionSource::SongRec,
        identity: TrackIdentity {
            title: trimmed.to_string(),
            artist: "Unknown Artist".to_string(),
            album: None,
            album_artist: None,
            duration_ms: None,
            isrc: None,
        },
        raw_json: Some(serde_json::json!({ "stdout": trimmed })),
        confidence: None,
    })
}
