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

    let path = args.path.clone();

    let identity = tokio::task::spawn_blocking(move || {
        let recognizer = SongRecRecognizer::default();
        recognizer.recognize_file(path)
    })
    .await??;

    let client = AppleMetadataClient::default();
    let mut candidates = client.search_track(&identity).await?;

    for candidate in &mut candidates {
        candidate.confidence = score_candidate(
            &identity,
            candidate,
            config.scoring.duration_tolerance_seconds,
        );
    }

    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

    println!("Recognized: {} - {}", identity.artist, identity.title);

    for candidate in candidates.iter().take(5) {
        println!(
            "{:.1} | {} - {} | {:?} | {:?}",
            candidate.confidence,
            candidate.artist,
            candidate.title,
            candidate.album,
            candidate.release_date
        );
    }

    Ok(())
}
