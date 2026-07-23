use clap::{Args, ValueEnum};
use std::fmt::Write as _;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use tunetagger_core::{AppConfig, TuneTaggerError};
use tunetagger_files::scan_mp3_files;

use super::tag::{run_with_outcome as tag_one, TagArgs, TagOutcome};

#[derive(Debug)]
struct BatchFailure {
    path: PathBuf,
    category: String,
    reason: String,
}

#[derive(Debug)]
struct BatchSkipped {
    source: PathBuf,
    destination: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ExistingPolicy {
    /// Ask what to do for each existing destination match
    Ask,
    /// Skip every source file with an existing destination match
    Skip,
    /// Process every source file even when a destination match exists
    Process,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExistingDecision {
    Skip,
    SkipAll,
    Process,
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

    /// How to handle files already present in the destination
    #[arg(long, value_enum, default_value_t = ExistingPolicy::Ask)]
    pub existing: ExistingPolicy,
}

pub async fn run(config_path: PathBuf, args: BatchArgs) -> anyhow::Result<()> {
    let _config = AppConfig::load(&config_path)?;
    let mut files = scan_mp3_files(&args.path, args.recursive)?;
    exclude_output_tree(&mut files, &args.output);

    println!("Found {} MP3 file(s)", files.len());

    let mut succeeded = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();
    let mut skip_all_existing = false;

    for file in files {
        let destination = destination_path(&args.output, &file.path);
        if destination.is_file() {
            let decision = if skip_all_existing {
                ExistingDecision::Skip
            } else {
                match args.existing {
                    ExistingPolicy::Ask => prompt_for_existing(&file.path, &destination)?,
                    ExistingPolicy::Skip => ExistingDecision::Skip,
                    ExistingPolicy::Process => ExistingDecision::Process,
                }
            };

            match decision {
                ExistingDecision::Skip | ExistingDecision::SkipAll => {
                    if decision == ExistingDecision::SkipAll {
                        skip_all_existing = true;
                    }

                    println!(
                        "Skipping {} (destination exists: {})",
                        file.path.display(),
                        destination.display()
                    );
                    skipped.push(BatchSkipped {
                        source: file.path,
                        destination,
                    });
                    continue;
                }
                ExistingDecision::Process => {}
            }
        }

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

    write_report(
        &args.report,
        &args.path,
        &args.output,
        &succeeded,
        &skipped,
        &failed,
    )?;

    println!();
    println!("Batch complete.");
    println!("  Successful: {}", succeeded.len());
    println!("  Skipped:    {}", skipped.len());
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

fn destination_path(output: &Path, source: &Path) -> PathBuf {
    output.join(source.file_name().unwrap_or_default())
}

fn exclude_output_tree(files: &mut Vec<tunetagger_core::AudioFile>, output: &Path) {
    let Ok(output) = std::fs::canonicalize(output) else {
        return;
    };

    files.retain(|file| {
        std::fs::canonicalize(&file.path)
            .map(|path| !path.starts_with(&output))
            .unwrap_or(true)
    });
}

fn prompt_for_existing(source: &Path, destination: &Path) -> anyhow::Result<ExistingDecision> {
    if !std::io::stdin().is_terminal() {
        anyhow::bail!(
            "destination file already exists: {}. Non-interactive input cannot use \
             '--existing ask'; choose '--existing skip' or '--existing process'",
            destination.display()
        );
    }

    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    prompt_for_existing_with_io(source, destination, &mut input, &mut output)
}

fn prompt_for_existing_with_io<R: BufRead, W: Write>(
    source: &Path,
    destination: &Path,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<ExistingDecision> {
    loop {
        writeln!(output, "\nDestination match found:")?;
        writeln!(output, "  Source:      {}", source.display())?;
        writeln!(output, "  Destination: {}", destination.display())?;
        write!(
            output,
            "Skip this file, skip all matches, or process it? [s/a/p]: "
        )?;
        output.flush()?;

        let mut answer = String::new();
        if input.read_line(&mut answer)? == 0 {
            anyhow::bail!("input ended before an existing-file choice was made");
        }

        if let Some(decision) = parse_existing_choice(&answer) {
            return Ok(decision);
        }

        writeln!(output, "Please enter 's', 'a', or 'p'.")?;
    }
}

fn parse_existing_choice(answer: &str) -> Option<ExistingDecision> {
    match answer.trim().to_ascii_lowercase().as_str() {
        "s" | "skip" => Some(ExistingDecision::Skip),
        "a" | "all" | "skip-all" => Some(ExistingDecision::SkipAll),
        "p" | "process" | "continue" => Some(ExistingDecision::Process),
        _ => None,
    }
}

fn write_report(
    report_path: &Path,
    input_path: &Path,
    output_path: &Path,
    succeeded: &[PathBuf],
    skipped: &[BatchSkipped],
    failed: &[BatchFailure],
) -> anyhow::Result<()> {
    if let Some(parent) = report_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut report = String::new();
    writeln!(report, "TUNETAGGER BATCH REPORT")?;
    writeln!(report, "=======================")?;
    writeln!(report, "Source:      {}", input_path.display())?;
    writeln!(report, "Destination: {}", output_path.display())?;
    writeln!(report)?;
    writeln!(report, "RESULTS")?;
    writeln!(report, "-------")?;
    writeln!(report, "Successful  {:>6}", succeeded.len())?;
    writeln!(report, "Skipped     {:>6}", skipped.len())?;
    writeln!(report, "Failed      {:>6}", failed.len())?;

    writeln!(report, "\nFAILED FILES ({})", failed.len())?;
    writeln!(
        report,
        "{}",
        "-".repeat(15 + failed.len().to_string().len())
    )?;
    if failed.is_empty() {
        writeln!(report, "None")?;
    } else {
        for (index, failure) in failed.iter().enumerate() {
            writeln!(
                report,
                "{}. {}",
                index + 1,
                display_relative(&failure.path, input_path)
            )?;
            writeln!(report, "   Category: {}", failure.category)?;
            writeln!(
                report,
                "   Reason:   {}",
                indent_report_reason(&failure.reason)
            )?;
        }
    }

    writeln!(report, "\nSKIPPED FILES ({})", skipped.len())?;
    writeln!(
        report,
        "{}",
        "-".repeat(16 + skipped.len().to_string().len())
    )?;
    if skipped.is_empty() {
        writeln!(report, "None")?;
    } else {
        writeln!(
            report,
            "These files were skipped because the destination filename already exists."
        )?;
        for (index, item) in skipped.iter().enumerate() {
            writeln!(
                report,
                "{}. {} -> {}",
                index + 1,
                display_relative(&item.source, input_path),
                display_relative(&item.destination, output_path)
            )?;
        }
    }

    writeln!(report, "\nSUCCESSFUL FILES ({})", succeeded.len())?;
    writeln!(
        report,
        "{}",
        "-".repeat(19 + succeeded.len().to_string().len())
    )?;
    if succeeded.is_empty() {
        writeln!(report, "None")?;
    } else {
        for (index, path) in succeeded.iter().enumerate() {
            writeln!(
                report,
                "{}. {}",
                index + 1,
                display_relative(path, input_path)
            )?;
        }
    }

    std::fs::write(report_path, report)?;
    Ok(())
}

fn display_relative(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn failure_category(error: &anyhow::Error) -> &'static str {
    for source in error.chain() {
        let Some(error) = source.downcast_ref::<TuneTaggerError>() else {
            continue;
        };

        return match error {
            TuneTaggerError::RecognitionNoMatch(_) => "recognition / no match",
            TuneTaggerError::RecognitionAudio(_) => "audio decoding",
            TuneTaggerError::RecognitionFingerprint(_) => "fingerprinting",
            TuneTaggerError::RecognitionService(_) => "recognition service",
            TuneTaggerError::Network(_) => "network",
            TuneTaggerError::Recognition(_) => "recognition",
            TuneTaggerError::Metadata(_) => "metadata",
            TuneTaggerError::Tagging(_) => "tagging",
            TuneTaggerError::ConfigParse(_) => "configuration",
            TuneTaggerError::Io(_) | TuneTaggerError::WalkDir(_) => "I/O",
            TuneTaggerError::UnsupportedFileType(_) | TuneTaggerError::Validation(_) => {
                "validation"
            }
        };
    }

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

fn indent_report_reason(reason: &str) -> String {
    reason.replace('\n', "\n             ")
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
    fn categorizes_structured_recognition_failures() {
        let no_match = anyhow::Error::new(TuneTaggerError::RecognitionNoMatch(
            "no matching track".to_owned(),
        ));
        let network =
            anyhow::Error::new(TuneTaggerError::Network("connection timed out".to_owned()));
        let audio = anyhow::Error::new(TuneTaggerError::RecognitionAudio(
            "could not decode stream".to_owned(),
        ));

        assert_eq!(failure_category(&no_match), "recognition / no match");
        assert_eq!(failure_category(&network), "network");
        assert_eq!(failure_category(&audio), "audio decoding");
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
        let skipped = vec![BatchSkipped {
            source: PathBuf::from("input/existing.mp3"),
            destination: PathBuf::from("output/existing.mp3"),
        }];
        let failed = vec![BatchFailure {
            path: PathBuf::from("input/bad.mp3"),
            category: "recognition".to_owned(),
            reason: "recognition failed: no match".to_owned(),
        }];

        write_report(
            &report_path,
            Path::new("input"),
            Path::new("output"),
            &succeeded,
            &skipped,
            &failed,
        )
        .unwrap();
        let report = std::fs::read_to_string(&report_path).unwrap();
        std::fs::remove_file(&report_path).unwrap();

        assert!(report.contains("Successful       1"));
        assert!(report.contains("Skipped          1"));
        assert!(report.contains("Failed           1"));
        assert!(report.contains("FAILED FILES (1)"));
        assert!(report.contains("1. bad.mp3"));
        assert!(report.contains("Category: recognition"));
        assert!(report.contains("Reason:   recognition failed: no match"));
        assert!(report.contains("SKIPPED FILES (1)"));
        assert!(report.contains("1. existing.mp3 -> existing.mp3"));
        assert!(report.contains("SUCCESSFUL FILES (1)"));
        assert!(report.contains("1. good.mp3"));
    }

    #[test]
    fn shortens_console_reasons() {
        let reason = "x".repeat(200);
        let shortened = brief_reason(&reason);
        assert_eq!(shortened.chars().count(), 160);
        assert!(shortened.ends_with("..."));
    }

    #[test]
    fn parses_each_existing_file_decision() {
        assert_eq!(parse_existing_choice("skip"), Some(ExistingDecision::Skip));
        assert_eq!(
            parse_existing_choice("skip-all"),
            Some(ExistingDecision::SkipAll)
        );
        assert_eq!(
            parse_existing_choice("process"),
            Some(ExistingDecision::Process)
        );
        assert_eq!(parse_existing_choice("unknown"), None);
    }

    #[test]
    fn prompts_again_after_an_invalid_existing_file_choice() {
        let mut input = std::io::Cursor::new(b"nope\np\n");
        let mut output = Vec::new();

        let decision = prompt_for_existing_with_io(
            Path::new("input/song.mp3"),
            Path::new("output/song.mp3"),
            &mut input,
            &mut output,
        )
        .unwrap();

        assert_eq!(decision, ExistingDecision::Process);
        assert!(String::from_utf8(output)
            .unwrap()
            .contains("Please enter 's', 'a', or 'p'."));
    }
}
