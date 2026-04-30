use clap::Args;
use std::path::PathBuf;
use tunetagger_core::AppConfig;
use tunetagger_recognition::SongRecRecognizer;

#[derive(Debug, Args)]
pub struct RecognizeArgs {
    pub path: PathBuf,
}

pub async fn run(config_path: PathBuf, args: RecognizeArgs) -> anyhow::Result<()> {
    let config = AppConfig::load(config_path)?;
    let recognizer = SongRecRecognizer::new(
        config.recognition.songrec_path,
        config.recognition.timeout_seconds,
    );
    let result = recognizer.recognize_file(args.path).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
