use std::path::{Path, PathBuf};
use tunetagger_core::{ProcessingAudit, TuneTaggerResult};

pub fn sidecar_path_for(input: impl AsRef<Path>) -> PathBuf {
    let input = input.as_ref();
    let mut sidecar = input.to_path_buf();
    sidecar.set_extension("tunetagger.json");
    sidecar
}

pub fn write_sidecar(
    input: impl AsRef<Path>,
    audit: &ProcessingAudit,
) -> TuneTaggerResult<PathBuf> {
    let path = sidecar_path_for(input);
    let json = serde_json::to_string_pretty(audit)
        .map_err(|err| tunetagger_core::TuneTaggerError::Validation(err.to_string()))?;
    std::fs::write(&path, json)?;
    Ok(path)
}
