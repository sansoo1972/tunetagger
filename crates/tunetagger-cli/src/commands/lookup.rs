use clap::Args;
use std::path::PathBuf;
use tunetagger_core::AppConfig;
use tunetagger_metadata::{scoring::score_candidate, AppleMetadataClient};
use tunetagger_recognition::SongRecRecognizer;

#[derive(Debug, Args)]
pub struct LookupArgs {
    pub path: PathBuf,
}

pub async fn run(config_path: PathBuf, args: LookupArgs) -> anyhow::Result<()> {
    let config = AppConfig::load(config_path)?;
    let recognizer = SongRecRecognizer::new(
        config.recognition.songrec_path,
        config.recognition.timeout_seconds,
    );
    let recognition = recognizer.recognize_file(&args.path).await?;
    let client = AppleMetadataClient::default();
    let mut candidates = client.search_track(&recognition.identity).await?;

    for candidate in &mut candidates {
        candidate.confidence = score_candidate(
            &recognition.identity,
            candidate,
            config.scoring.duration_tolerance_seconds,
        );
    }
    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

    println!(
        "Recognized: {} - {}",
        recognition.identity.artist, recognition.identity.title
    );
    println!("Candidates:");
    for candidate in candidates {
        println!(
            "{:>5.1}  {} - {}  [{}]",
            candidate.confidence,
            candidate.artist,
            candidate.title,
            candidate
                .album
                .unwrap_or_else(|| "unknown album".to_string())
        );
    }
    Ok(())
}
