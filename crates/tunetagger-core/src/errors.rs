use thiserror::Error;

pub type TuneTaggerResult<T> = Result<T, TuneTaggerError>;

#[derive(Debug, Error)]
pub enum TuneTaggerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("recognition failed: {0}")]
    Recognition(String),

    #[error("metadata lookup failed: {0}")]
    Metadata(String),

    #[error("tagging failed: {0}")]
    Tagging(String),

    #[error("unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("validation failed: {0}")]
    Validation(String),
}
