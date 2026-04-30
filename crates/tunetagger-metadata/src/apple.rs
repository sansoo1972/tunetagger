use serde::Deserialize;
use tunetagger_core::{
    MetadataCandidate, MetadataSource, TrackIdentity, TuneTaggerError, TuneTaggerResult,
};

#[derive(Debug, Clone)]
pub struct AppleMetadataClient {
    client: reqwest::Client,
}

impl Default for AppleMetadataClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl AppleMetadataClient {
    pub async fn search_track(
        &self,
        identity: &TrackIdentity,
    ) -> TuneTaggerResult<Vec<MetadataCandidate>> {
        let search_term = format!("{} {}", identity.artist, identity.title);
        let term = urlencoding::encode(&search_term);
        let url =
            format!("https://itunes.apple.com/search?media=music&entity=song&limit=10&term={term}");

        let response: AppleSearchResponse = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?
            .json()
            .await
            .map_err(|err| TuneTaggerError::Metadata(err.to_string()))?;

        Ok(response.results.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Deserialize)]
struct AppleSearchResponse {
    results: Vec<AppleTrack>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleTrack {
    track_name: Option<String>,
    artist_name: Option<String>,
    collection_name: Option<String>,
    collection_artist_name: Option<String>,
    track_number: Option<u16>,
    track_count: Option<u16>,
    disc_number: Option<u16>,
    disc_count: Option<u16>,
    release_date: Option<String>,
    primary_genre_name: Option<String>,
    track_explicitness: Option<String>,
    artwork_url100: Option<String>,
    track_time_millis: Option<u64>,
}

impl From<AppleTrack> for MetadataCandidate {
    fn from(value: AppleTrack) -> Self {
        let artwork_url = value
            .artwork_url100
            .map(|u| u.replace("100x100bb", "1200x1200bb"));
        MetadataCandidate {
            source: MetadataSource::Apple,
            title: value.track_name.unwrap_or_default(),
            artist: value.artist_name.unwrap_or_default(),
            album: value.collection_name,
            album_artist: value.collection_artist_name,
            track_number: value.track_number,
            track_total: value.track_count,
            disc_number: value.disc_number,
            disc_total: value.disc_count,
            release_date: value.release_date,
            genre: value.primary_genre_name,
            composer: None,
            explicit: value.track_explicitness.map(|e| e == "explicit"),
            artwork_url,
            duration_ms: value.track_time_millis,
            confidence: 0.0,
        }
    }
}
