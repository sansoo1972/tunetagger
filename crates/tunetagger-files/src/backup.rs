use std::path::{Path, PathBuf};
use tunetagger_core::TuneTaggerResult;

pub fn backup_file(
    input: impl AsRef<Path>,
    archive_dir: impl AsRef<Path>,
) -> TuneTaggerResult<PathBuf> {
    let input = input.as_ref();
    std::fs::create_dir_all(&archive_dir)?;
    let file_name = input.file_name().unwrap_or_default();
    let destination = archive_dir.as_ref().join(file_name);
    std::fs::copy(input, &destination)?;
    Ok(destination)
}
