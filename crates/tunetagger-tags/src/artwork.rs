use tunetagger_core::{TuneTaggerError, TuneTaggerResult};

pub async fn download_artwork(url: &str) -> TuneTaggerResult<(Vec<u8>, String)> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?;

    let mime_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?;

    Ok((bytes.to_vec(), mime_type))
}
