use clap::Args;
use std::path::PathBuf;
use tunetagger_core::{AppConfig, ResolvedTags};
use tunetagger_metadata::{scoring::score_candidate, AppleMetadataClient};
use tunetagger_recognition::SongRecRecognizer;
use tunetagger_tags::{download_artwork, Id3Mp3TagWriter};

#[derive(Debug, Args)]
pub struct TagArgs {
    pub path: PathBuf,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub write: bool,

    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn run(config_path: PathBuf, args: TagArgs) -> anyhow::Result<()> {
    let config = AppConfig::load(config_path)?;
    let existing = Id3Mp3TagWriter::read_tags(&args.path)?;

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
            config.scoring.duration_tolerance_seconds.into(),
        );
    }
    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

    let Some(best) = candidates.first() else {
        println!("No metadata candidates found.");
        return Ok(());
    };

    let proposed = ResolvedTags {
        title: best.title.clone(),
        artist: best.artist.clone(),
        album: best.album.clone(),
        album_artist: best.album_artist.clone(),
        track_number: best.track_number,
        track_total: best.track_total,
        disc_number: best.disc_number,
        disc_total: best.disc_total,
        release_date: best.release_date.clone(),
        genre: best.genre.clone(),
        composer: best.composer.clone(),
        sort_artist: None,
        sort_album: None,
        sort_album_artist: None,
        lyrics: None,
    };

    println!("Best candidate score: {:.1}", best.confidence);
    println!("Proposed tags:");
    println!("  Title:  {:?} -> {}", existing.title, proposed.title);
    println!("  Artist: {:?} -> {}", existing.artist, proposed.artist);
    println!("  Album:  {:?} -> {:?}", existing.album, proposed.album);
    println!(
        "  Track:  {:?} -> {:?}",
        existing.track_number, proposed.track_number
    );
    println!("  Genre:  {:?} -> {:?}", existing.genre, proposed.genre);

    if args.write && !args.dry_run {
        let target = if let Some(output_dir) = args.output {
            std::fs::create_dir_all(&output_dir)?;
            let file_name = args.path.file_name().unwrap_or_default();
            let target = output_dir.join(file_name);
            std::fs::copy(&args.path, &target)?;
            target
        } else {
            args.path.clone()
        };

        Id3Mp3TagWriter::write_tags(&target, &proposed)?;

        if let Some(artwork_url) = &best.artwork_url {
            println!("Downloading artwork: {artwork_url}");

            match download_artwork(artwork_url).await {
                Ok(artwork) => {
                    Id3Mp3TagWriter::embed_artwork(&target, &artwork.bytes, &artwork.mime_type)?;
                    println!("Embedded artwork.");
                }
                Err(err) => {
                    eprintln!("Warning: could not embed artwork: {err}");
                }
            }
        } else {
            println!("No artwork URL found for best candidate.");
        }

        println!("Wrote tags to {}", target.display());
    } else {
        println!("Dry run only. Use --write without --dry-run to modify an output copy or file.");
    }

    Ok(())
}
