use serde::Deserialize;
use tunetagger_core::{MetadataCandidate, TrackIdentity, TuneTaggerError, TuneTaggerResult};

#[derive(Debug, Clone)]
pub struct MusicBrainzClient {
    client: reqwest::Client,
    user_agent: String,
}

#[derive(Debug, Clone, Default)]
pub struct MusicBrainzEnrichment {
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub release_title: Option<String>,
    pub release_id: Option<String>,
    pub recording_id: Option<String>,
}

impl Default for MusicBrainzClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            user_agent: "TuneTagger/0.1.2 (https://github.com/sansoo1972/tunetagger)".to_string(),
        }
    }
}

impl MusicBrainzClient {
    pub async fn enrich(
        &self,
        candidate: &MetadataCandidate,
    ) -> TuneTaggerResult<MusicBrainzEnrichment> {
        let Some(recording) = self.find_best_recording(candidate).await? else {
            return Ok(MusicBrainzEnrichment::default());
        };

        let recording_id = recording.id.clone();

        let release = self
            .find_best_release_for_recording(&recording_id, candidate)
            .await?;

        let album_artist = release
            .as_ref()
            .and_then(MusicBrainzRelease::artist_credit_name);

        let release_title = release.as_ref().map(|release| release.title.clone());
        let release_id = release.as_ref().map(|release| release.id.clone());

        let composer = self.lookup_recording_composer(&recording_id).await?;

        Ok(MusicBrainzEnrichment {
            album_artist,
            composer,
            release_title,
            release_id,
            recording_id: Some(recording_id),
        })
    }

    pub async fn enrich_composer(
        &self,
        candidate: &MetadataCandidate,
    ) -> TuneTaggerResult<Option<String>> {
        Ok(self.enrich(candidate).await?.composer)
    }

    async fn find_best_recording(
        &self,
        candidate: &MetadataCandidate,
    ) -> TuneTaggerResult<Option<MusicBrainzRecording>> {
        let query = build_recording_query(candidate);
        let encoded = urlencoding::encode(&query);

        let url =
            format!("https://musicbrainz.org/ws/2/recording?query={encoded}&fmt=json&limit=10");

        let response: RecordingSearchResponse = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!("MusicBrainz recording search failed: {err}"))
            })?
            .json()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!(
                    "MusicBrainz recording search JSON decode failed: {err}"
                ))
            })?;

        let candidate_title = normalize(&candidate.title);
        let candidate_artist = normalize(&candidate.artist);

        let best = response
            .recordings
            .into_iter()
            .filter(|recording| normalize(&recording.title) == candidate_title)
            .filter(|recording| {
                recording.artist_credit.iter().any(|credit| {
                    let credited_name_matches = normalize(&credit.name) == candidate_artist;

                    let artist_name_matches = credit
                        .artist
                        .as_ref()
                        .map(|artist| normalize(&artist.name) == candidate_artist)
                        .unwrap_or(false);

                    credited_name_matches || artist_name_matches
                })
            })
            .max_by_key(|recording| recording.score.unwrap_or_default());

        Ok(best)
    }

    async fn find_best_release_for_recording(
        &self,
        recording_id: &str,
        candidate: &MetadataCandidate,
    ) -> TuneTaggerResult<Option<MusicBrainzRelease>> {
        let url = format!(
            "https://musicbrainz.org/ws/2/recording/{recording_id}?inc=releases+artist-credits&fmt=json"
        );

        let response: RecordingReleaseLookupResponse = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!("MusicBrainz release lookup failed: {err}"))
            })?
            .json()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!(
                    "MusicBrainz release lookup JSON decode failed: {err}"
                ))
            })?;

        let mut releases = response.releases.unwrap_or_default();

        let Some(candidate_album) = candidate.album.as_ref().map(|album| normalize(album)) else {
            return Ok(releases.into_iter().next());
        };

        releases.sort_by_key(|release| {
            let release_title = normalize(&release.title);

            if release_title == candidate_album {
                0
            } else if release_title.contains(&candidate_album)
                || candidate_album.contains(&release_title)
            {
                1
            } else {
                2
            }
        });

        Ok(releases.into_iter().next())
    }

    async fn lookup_recording_composer(
        &self,
        recording_id: &str,
    ) -> TuneTaggerResult<Option<String>> {
        let url = format!(
            "https://musicbrainz.org/ws/2/recording/{recording_id}?inc=work-rels+artist-rels&fmt=json"
        );

        let response: RecordingLookupResponse = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!("MusicBrainz recording lookup failed: {err}"))
            })?
            .json()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!(
                    "MusicBrainz recording lookup JSON decode failed: {err}"
                ))
            })?;

        let mut composers = Vec::new();

        for relation in response.relations.unwrap_or_default() {
            if relation.relation_type == "composer" {
                if let Some(artist) = relation.artist {
                    push_non_empty(&mut composers, artist.name);
                }
            }

            if matches!(
                relation.relation_type.as_str(),
                "performance" | "recording of"
            ) {
                if let Some(work) = relation.work {
                    let work_composers = self.lookup_work_composers(&work.id).await?;
                    composers.extend(work_composers);
                }
            }
        }

        normalize_name_list(composers)
    }

    async fn lookup_work_composers(&self, work_id: &str) -> TuneTaggerResult<Vec<String>> {
        let url = format!("https://musicbrainz.org/ws/2/work/{work_id}?inc=artist-rels&fmt=json");

        let response: WorkLookupResponse = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!("MusicBrainz work lookup failed: {err}"))
            })?
            .json()
            .await
            .map_err(|err| {
                TuneTaggerError::Metadata(format!(
                    "MusicBrainz work lookup JSON decode failed: {err}"
                ))
            })?;

        let mut composers = Vec::new();

        for relation in response.relations.unwrap_or_default() {
            if matches!(
                relation.relation_type.as_str(),
                "composer" | "writer" | "lyricist"
            ) {
                if let Some(artist) = relation.artist {
                    push_non_empty(&mut composers, artist.name);
                }
            }
        }

        composers.sort();
        composers.dedup();

        Ok(composers)
    }
}

pub fn infer_album_artist_only_if_safe(
    identity: &TrackIdentity,
    candidate: &MetadataCandidate,
) -> Option<String> {
    let artist = candidate.artist.trim();

    if artist.is_empty() {
        return None;
    }

    let candidate_album = candidate.album.as_deref()?;

    let album = candidate_album.to_lowercase();
    let artist_lower = artist.to_lowercase();

    let risky_album = [
        "soundtrack",
        "original motion picture",
        "various artists",
        "compilation",
        "tribute",
        "hits of",
        "best of the",
        "now that's what i call",
        "now thats what i call",
        "motion picture",
        "movie",
        "banda sonora",
        "original soundtrack",
        "music from",
    ]
    .iter()
    .any(|needle| album.contains(needle));

    let risky_artist = [
        " feat. ",
        " ft. ",
        " featuring ",
        " & ",
        " and ",
        ",",
        " x ",
        " vs. ",
        " with ",
        " con ",
    ]
    .iter()
    .any(|needle| artist_lower.contains(needle));

    if risky_album || risky_artist {
        return None;
    }

    let identity_album_matches = identity
        .album
        .as_deref()
        .map(|identity_album| normalize(identity_album) == normalize(candidate_album))
        .unwrap_or(false);

    if identity_album_matches {
        Some(artist.to_string())
    } else {
        None
    }
}

fn build_recording_query(candidate: &MetadataCandidate) -> String {
    let mut parts = vec![
        format!("recording:\"{}\"", candidate.title),
        format!("artist:\"{}\"", candidate.artist),
    ];

    if let Some(album) = &candidate.album {
        parts.push(format!("release:\"{}\"", album));
    }

    parts.join(" AND ")
}

fn push_non_empty(values: &mut Vec<String>, value: String) {
    let trimmed = value.trim();

    if !trimmed.is_empty() {
        values.push(trimmed.to_string());
    }
}

fn normalize_name_list(mut names: Vec<String>) -> TuneTaggerResult<Option<String>> {
    names = names
        .into_iter()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();

    names.sort();
    names.dedup();

    if names.is_empty() {
        Ok(None)
    } else {
        Ok(Some(names.join("; ")))
    }
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace('&', "and")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Deserialize)]
struct RecordingSearchResponse {
    #[serde(default)]
    recordings: Vec<MusicBrainzRecording>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MusicBrainzRecording {
    #[serde(default)]
    id: String,

    #[serde(default)]
    title: String,

    #[serde(default)]
    score: Option<u16>,

    #[serde(default)]
    artist_credit: Vec<MusicBrainzArtistCredit>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzArtistCredit {
    #[serde(default)]
    name: String,

    #[serde(default)]
    artist: Option<MusicBrainzArtist>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzArtist {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct RecordingReleaseLookupResponse {
    #[serde(default)]
    releases: Option<Vec<MusicBrainzRelease>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MusicBrainzRelease {
    #[serde(default)]
    id: String,

    #[serde(default)]
    title: String,

    #[serde(default)]
    artist_credit: Vec<MusicBrainzArtistCredit>,
}

impl MusicBrainzRelease {
    fn artist_credit_name(&self) -> Option<String> {
        let names = self
            .artist_credit
            .iter()
            .filter_map(|credit| {
                if !credit.name.trim().is_empty() {
                    Some(credit.name.trim().to_string())
                } else {
                    credit
                        .artist
                        .as_ref()
                        .map(|artist| artist.name.trim().to_string())
                        .filter(|name| !name.is_empty())
                }
            })
            .collect::<Vec<_>>();

        if names.is_empty() {
            None
        } else {
            Some(names.join(" & "))
        }
    }
}

#[derive(Debug, Deserialize)]
struct RecordingLookupResponse {
    #[serde(default)]
    relations: Option<Vec<MusicBrainzRelation>>,
}

#[derive(Debug, Deserialize)]
struct WorkLookupResponse {
    #[serde(default)]
    relations: Option<Vec<MusicBrainzRelation>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MusicBrainzRelation {
    #[serde(rename = "type")]
    #[serde(default)]
    relation_type: String,

    #[serde(default)]
    artist: Option<MusicBrainzArtist>,

    #[serde(default)]
    work: Option<MusicBrainzWork>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzWork {
    #[serde(default)]
    id: String,
}
