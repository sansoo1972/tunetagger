use clap::Args;
use std::path::PathBuf;
use tunetagger_recognition::SongRecRecognizer;

#[derive(Debug, Args)]
pub struct RecognizeArgs {
    pub path: PathBuf,
}

pub async fn run(_config_path: PathBuf, args: RecognizeArgs) -> anyhow::Result<()> {
    let path = args.path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let recognizer = SongRecRecognizer::default();
        recognizer.recognize_file(path)
    })
    .await??;

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
