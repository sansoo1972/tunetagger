mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "tunetagger")]
#[command(about = "Identify MP3 files and write clean ID3 metadata", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "config/tunetagger.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Scan(commands::scan::ScanArgs),
    Recognize(commands::recognize::RecognizeArgs),
    Lookup(commands::lookup::LookupArgs),
    Tag(commands::tag::TagArgs),
    Batch(commands::batch::BatchArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Scan(args) => commands::scan::run(args).await?,
        Command::Recognize(args) => commands::recognize::run(cli.config, args).await?,
        Command::Lookup(args) => commands::lookup::run(cli.config, args).await?,
        Command::Tag(args) => commands::tag::run(cli.config, args).await?,
        Command::Batch(args) => commands::batch::run(cli.config, args).await?,
    }

    Ok(())
}
