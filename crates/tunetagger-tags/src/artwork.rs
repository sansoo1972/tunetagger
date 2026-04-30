use tunetagger_core::{TuneTaggerError, TuneTaggerResult};

#[derive(Debug, Clone)]
pub struct DownloadedArtwork {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

pub async fn download_artwork(url: &str) -> TuneTaggerResult<DownloadedArtwork> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?;

    if !response.status().is_success() {
        return Err(TuneTaggerError::Metadata(format!(
            "artwork download failed with HTTP status {}",
            response.status()
        )));
    }

    let mime_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?
        .to_vec();

    Ok(DownloadedArtwork { bytes, mime_type })
}
