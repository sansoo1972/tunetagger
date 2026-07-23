use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use super::batch::{self, BatchArgs, ExistingPolicy};

pub async fn run(config_path: PathBuf) -> anyhow::Result<()> {
    if !std::io::stdin().is_terminal() {
        anyhow::bail!(
            "interactive setup requires a terminal; use a TuneTagger subcommand and CLI options \
             for non-interactive runs"
        );
    }

    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut output = stdout.lock();

    if let Some(args) = collect_batch_args(&mut input, &mut output)? {
        drop(input);
        drop(output);
        batch::run(config_path, args).await?;
    }

    Ok(())
}

fn collect_batch_args<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<Option<BatchArgs>> {
    writeln!(output, "TuneTagger guided setup")?;
    writeln!(output, "=======================")?;
    writeln!(output, "1. Batch-tag an MP3 folder")?;
    writeln!(output, "q. Quit")?;

    loop {
        let operation = prompt(input, output, "Choose an operation [1]: ")?;
        match operation.trim().to_ascii_lowercase().as_str() {
            "" | "1" | "batch" => break,
            "q" | "quit" => {
                writeln!(output, "Canceled. No files were changed.")?;
                return Ok(None);
            }
            _ => writeln!(output, "Please enter '1' or 'q'.")?,
        }
    }

    let source = prompt_existing_directory(input, output)?;
    let default_destination = source.join("batch_tagged");
    let destination = prompt_path(
        input,
        output,
        &format!("Destination folder [{}]: ", default_destination.display()),
        Some(&default_destination),
    )?;

    show_path_warning(output, &source, &destination)?;

    let recursive = prompt_yes_no(
        input,
        output,
        "Include MP3 files from subfolders? [y/N]: ",
        false,
    )?;
    let (dry_run, write) = prompt_run_mode(input, output)?;
    let existing = prompt_existing_policy(input, output)?;
    let report = prompt_path(
        input,
        output,
        "Report file [batch-report.txt]: ",
        Some(Path::new("batch-report.txt")),
    )?;

    writeln!(output, "\nReview settings")?;
    writeln!(output, "---------------")?;
    writeln!(output, "Source:             {}", source.display())?;
    writeln!(output, "Destination:        {}", destination.display())?;
    writeln!(
        output,
        "Include subfolders: {}",
        if recursive { "yes" } else { "no" }
    )?;
    writeln!(
        output,
        "Mode:               {}",
        if write {
            "write tagged copies"
        } else {
            "dry run"
        }
    )?;
    writeln!(output, "Existing files:     {}", policy_name(existing))?;
    writeln!(output, "Report:             {}", report.display())?;

    if !prompt_yes_no(input, output, "Start with these settings? [y/N]: ", false)? {
        writeln!(output, "Canceled. No files were changed.")?;
        return Ok(None);
    }

    Ok(Some(BatchArgs {
        path: source,
        output: destination,
        recursive,
        write,
        dry_run,
        report,
        existing,
    }))
}

fn prompt_existing_directory<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<PathBuf> {
    loop {
        let path = prompt_path(input, output, "Source folder: ", None)?;
        if path.is_dir() {
            return Ok(path);
        }
        writeln!(
            output,
            "That source folder does not exist or is not a directory."
        )?;
    }
}

fn prompt_path<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    message: &str,
    default: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    loop {
        let value = prompt(input, output, message)?;
        if value.trim().is_empty() {
            if let Some(default) = default {
                return Ok(default.to_path_buf());
            }
            writeln!(output, "A path is required.")?;
            continue;
        }
        return Ok(expand_tilde(value.trim()));
    }
}

fn prompt_run_mode<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<(bool, bool)> {
    loop {
        let value = prompt(
            input,
            output,
            "Run mode: dry run or write tagged copies? [d/w, default d]: ",
        )?;
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "d" | "dry" | "dry-run" => return Ok((true, false)),
            "w" | "write" => return Ok((false, true)),
            _ => writeln!(output, "Please enter 'd' for dry run or 'w' for write.")?,
        }
    }
}

fn prompt_existing_policy<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<ExistingPolicy> {
    loop {
        let value = prompt(
            input,
            output,
            "When a destination file exists: ask, skip, or process? [a/s/p, default a]: ",
        )?;
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "a" | "ask" => return Ok(ExistingPolicy::Ask),
            "s" | "skip" => return Ok(ExistingPolicy::Skip),
            "p" | "process" => return Ok(ExistingPolicy::Process),
            _ => writeln!(output, "Please enter 'a', 's', or 'p'.")?,
        }
    }
}

fn prompt_yes_no<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    message: &str,
    default: bool,
) -> anyhow::Result<bool> {
    loop {
        let value = prompt(input, output, message)?;
        match value.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => writeln!(output, "Please enter 'y' or 'n'.")?,
        }
    }
}

fn prompt<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    message: &str,
) -> anyhow::Result<String> {
    write!(output, "{message}")?;
    output.flush()?;

    let mut value = String::new();
    if input.read_line(&mut value)? == 0 {
        anyhow::bail!("input ended before interactive setup was complete");
    }
    Ok(value)
}

fn expand_tilde(value: &str) -> PathBuf {
    if value == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(value));
    }

    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(value)
}

fn show_path_warning<W: Write>(
    output: &mut W,
    source: &Path,
    destination: &Path,
) -> anyhow::Result<()> {
    if source == destination {
        writeln!(
            output,
            "Warning: source and destination are the same folder; write mode may replace files."
        )?;
    } else if destination.starts_with(source) {
        writeln!(
            output,
            "Note: the destination is inside the source. TuneTagger will exclude it from recursive scans."
        )?;
    }
    Ok(())
}

fn policy_name(policy: ExistingPolicy) -> &'static str {
    match policy {
        ExistingPolicy::Ask => "ask case by case",
        ExistingPolicy::Skip => "skip",
        ExistingPolicy::Process => "process",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_safe_guided_batch_settings() {
        let test_root = std::env::temp_dir().join(format!(
            "tunetagger-interactive-{}-collect",
            std::process::id()
        ));
        std::fs::create_dir_all(&test_root).unwrap();
        let destination = test_root.join("tagged output");
        let input_text = format!(
            "1\n{}\n{}\nn\nd\ns\nreport file.txt\ny\n",
            test_root.display(),
            destination.display()
        );
        let mut input = std::io::Cursor::new(input_text);
        let mut output = Vec::new();

        let args = collect_batch_args(&mut input, &mut output)
            .unwrap()
            .expect("settings should be confirmed");

        std::fs::remove_dir_all(&test_root).unwrap();
        assert_eq!(args.path, test_root);
        assert_eq!(args.output, destination);
        assert!(!args.recursive);
        assert!(args.dry_run);
        assert!(!args.write);
        assert_eq!(args.existing, ExistingPolicy::Skip);
        assert_eq!(args.report, PathBuf::from("report file.txt"));
    }

    #[test]
    fn cancellation_returns_no_batch_settings() {
        let mut input = std::io::Cursor::new(b"q\n");
        let mut output = Vec::new();

        let args = collect_batch_args(&mut input, &mut output).unwrap();

        assert!(args.is_none());
        assert!(String::from_utf8(output)
            .unwrap()
            .contains("No files were changed"));
    }

    #[test]
    fn expands_home_directory_paths() {
        if let Some(home) = std::env::var_os("HOME") {
            assert_eq!(expand_tilde("~/Music"), PathBuf::from(home).join("Music"));
        }
    }
}
