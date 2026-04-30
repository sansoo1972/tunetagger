use std::path::Path;
use tunetagger_core::{Lyrics, TuneTaggerResult};

pub fn read_lyrics_file(
    path: impl AsRef<Path>,
    language: impl Into<String>,
) -> TuneTaggerResult<Lyrics> {
    let text = std::fs::read_to_string(path)?;
    Ok(Lyrics {
        language: language.into(),
        description: None,
        text,
    })
}
