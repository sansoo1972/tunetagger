use clap::Args;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use tunetagger_core::AppConfig;
use tunetagger_files::scan_mp3_files;

use super::tag::{run_with_outcome as tag_one, TagArgs, TagOutcome};

#[derive(Debug)]
struct BatchFailure {
    path: PathBuf,
    category: String,
    reason: String,
}

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

    /// Path for the detailed plain-text batch report
    #[arg(long, default_value = "batch-report.txt")]
    pub report: PathBuf,
}

pub async fn run(config_path: PathBuf, args: BatchArgs) -> anyhow::Result<()> {
    let _config = AppConfig::load(&config_path)?;
    let files = scan_mp3_files(&args.path, args.recursive)?;

    println!("Found {} MP3 file(s)", files.len());

    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

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
            Ok(TagOutcome::Completed) => {
                succeeded.push(file.path);
            }
            Ok(TagOutcome::NoMetadataCandidates) => {
                let err = anyhow::anyhow!("metadata lookup failed: no candidates found");
                eprintln!("Failed: {}: {err}", file.path.display());
                failed.push(BatchFailure {
                    path: file.path,
                    category: failure_category(&err).to_owned(),
                    reason: err.to_string(),
                });
            }
            Err(err) => {
                eprintln!("Failed: {}: {err}", file.path.display());
                failed.push(BatchFailure {
                    path: file.path,
                    category: failure_category(&err).to_owned(),
                    reason: format!("{err:#}"),
                });
            }
        }
    }

    write_report(&args.report, &args.path, &succeeded, &failed)?;

    println!();
    println!("Batch complete.");
    println!("  Successful: {}", succeeded.len());
    println!("  Failed:     {}", failed.len());

    if !failed.is_empty() {
        println!("  Failure diagnostics:");
        for failure in &failed {
            println!(
                "    [{}] {}: {}",
                failure.category,
                failure.path.display(),
                brief_reason(&failure.reason)
            );
        }
    }

    println!("  Report:     {}", args.report.display());

    Ok(())
}

fn write_report(
    report_path: &Path,
    input_path: &Path,
    succeeded: &[PathBuf],
    failed: &[BatchFailure],
) -> anyhow::Result<()> {
    if let Some(parent) = report_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut report = String::new();
    writeln!(report, "TuneTagger Batch Report")?;
    writeln!(report, "Input: {}", input_path.display())?;
    writeln!(report, "Successful: {}", succeeded.len())?;
    writeln!(report, "Failed: {}", failed.len())?;

    writeln!(report, "\nSUCCEEDED")?;
    if succeeded.is_empty() {
        writeln!(report, "(none)")?;
    } else {
        for path in succeeded {
            writeln!(report, "[OK] {}", path.display())?;
        }
    }

    writeln!(report, "\nFAILED")?;
    if failed.is_empty() {
        writeln!(report, "(none)")?;
    } else {
        for failure in failed {
            writeln!(
                report,
                "[{}] {}",
                failure.category.to_uppercase(),
                failure.path.display()
            )?;
            writeln!(report, "  Reason: {}", indent_continuation(&failure.reason))?;
        }
    }

    std::fs::write(report_path, report)?;
    Ok(())
}

fn failure_category(error: &anyhow::Error) -> &'static str {
    let message = format!("{error:#}").to_ascii_lowercase();

    if message.contains("recognition") || message.contains("fingerprint") {
        "recognition"
    } else if message.contains("metadata") || message.contains("candidate") {
        "metadata"
    } else if message.contains("tagging") || message.contains("id3") {
        "tagging"
    } else if message.contains("config") {
        "configuration"
    } else if message.contains("network")
        || message.contains("http")
        || message.contains("request")
        || message.contains("connection")
    {
        "network"
    } else if error.chain().any(|source| source.is::<std::io::Error>()) {
        "I/O"
    } else {
        "processing"
    }
}

fn brief_reason(reason: &str) -> String {
    const MAX_CHARS: usize = 160;
    let one_line = reason.split_whitespace().collect::<Vec<_>>().join(" ");

    if one_line.chars().count() <= MAX_CHARS {
        return one_line;
    }

    let mut shortened = one_line.chars().take(MAX_CHARS - 3).collect::<String>();
    shortened.push_str("...");
    shortened
}

fn indent_continuation(reason: &str) -> String {
    reason.replace('\n', "\n          ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorizes_recognition_failures() {
        let error = anyhow::anyhow!("recognition failed: no fingerprint match");
        assert_eq!(failure_category(&error), "recognition");
    }

    #[test]
    fn categorizes_io_failures_from_error_chain() {
        let error = anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
        assert_eq!(failure_category(&error), "I/O");
    }

    #[test]
    fn renders_successes_failures_and_reasons() {
        let unique = format!(
            "tunetagger-batch-report-{}-{}.txt",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        );
        let report_path = std::env::temp_dir().join(unique);
        let succeeded = vec![PathBuf::from("input/good.mp3")];
        let failed = vec![BatchFailure {
            path: PathBuf::from("input/bad.mp3"),
            category: "recognition".to_owned(),
            reason: "recognition failed: no match".to_owned(),
        }];

        write_report(&report_path, Path::new("input"), &succeeded, &failed).unwrap();
        let report = std::fs::read_to_string(&report_path).unwrap();
        std::fs::remove_file(&report_path).unwrap();

        assert!(report.contains("Successful: 1"));
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("[OK] input/good.mp3"));
        assert!(report.contains("[RECOGNITION] input/bad.mp3"));
        assert!(report.contains("Reason: recognition failed: no match"));
    }

    #[test]
    fn shortens_console_reasons() {
        let reason = "x".repeat(200);
        let shortened = brief_reason(&reason);
        assert_eq!(shortened.chars().count(), 160);
        assert!(shortened.ends_with("..."));
    }
}
