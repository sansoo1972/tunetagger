use clap::Args;
use std::path::PathBuf;
use tunetagger_core::AppConfig;
use tunetagger_files::scan_mp3_files;

use super::tag::{run as tag_one, TagArgs};

#[derive(Debug, Args)]
pub struct BatchArgs {
    pub path: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(long)]
    pub recursive: bool,

    #[arg(long)]
    pub write: bool,

    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(config_path: PathBuf, args: BatchArgs) -> anyhow::Result<()> {
    let _config = AppConfig::load(&config_path)?;
    let files = scan_mp3_files(&args.path, args.recursive)?;

    println!("Found {} MP3 file(s)", files.len());

    let mut success_count = 0usize;
    let mut failure_count = 0usize;

    for file in files {
        println!();
        println!("Processing {}", file.path.display());

        let tag_args = TagArgs {
            path: file.path.clone(),
            dry_run: args.dry_run,
            write: args.write,
            output: Some(args.output.clone()),
        };

        match tag_one(config_path.clone(), tag_args).await {
            Ok(()) => {
                success_count += 1;
            }
            Err(err) => {
                failure_count += 1;
                eprintln!("Failed: {}: {err}", file.path.display());
            }
        }
    }

    println!();
    println!("Batch complete.");
    println!("  Successful: {success_count}");
    println!("  Failed:     {failure_count}");

    Ok(())
}
