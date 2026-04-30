use std::path::Path;
use tunetagger_core::{AudioFile, TuneTaggerResult};
use walkdir::WalkDir;

pub fn scan_mp3_files(root: impl AsRef<Path>, recursive: bool) -> TuneTaggerResult<Vec<AudioFile>> {
    let root = root.as_ref();
    let mut files = Vec::new();

    if root.is_file() {
        if is_mp3(root) {
            let metadata = std::fs::metadata(root)?;
            files.push(AudioFile {
                path: root.to_path_buf(),
                extension: "mp3".to_string(),
                size_bytes: metadata.len(),
            });
        }
        return Ok(files);
    }

    let walker = if recursive {
        WalkDir::new(root)
    } else {
        WalkDir::new(root).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && is_mp3(path) {
            let metadata = entry.metadata()?;
            files.push(AudioFile {
                path: path.to_path_buf(),
                extension: "mp3".to_string(),
                size_bytes: metadata.len(),
            });
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn is_mp3(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("mp3"))
        .unwrap_or(false)
}
