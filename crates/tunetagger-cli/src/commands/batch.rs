use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct BatchArgs {
    pub path: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(short, long)]
    pub recursive: bool,

    #[arg(long)]
    pub write: bool,
}

pub async fn run(_config_path: PathBuf, args: BatchArgs) -> anyhow::Result<()> {
    let files = tunetagger_files::scan_mp3_files(&args.path, args.recursive)?;
    println!("Batch foundation: {} MP3 file(s) queued", files.len());
    println!("Output: {}", args.output.display());
    println!("Write enabled: {}", args.write);
    println!("Full batch pipeline will be implemented after single-file tag flow is validated.");
    Ok(())
}
