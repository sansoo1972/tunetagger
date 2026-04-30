use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ScanArgs {
    pub path: PathBuf,

    #[arg(short, long)]
    pub recursive: bool,
}

pub async fn run(args: ScanArgs) -> anyhow::Result<()> {
    let files = tunetagger_files::scan_mp3_files(&args.path, args.recursive)?;
    println!("Found {} MP3 file(s)", files.len());
    for file in files {
        println!("{}  {} bytes", file.path.display(), file.size_bytes);
    }
    Ok(())
}
