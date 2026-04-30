use tunetagger_core::{MetadataCandidate, TrackIdentity};

pub fn score_candidate(
    identity: &TrackIdentity,
    candidate: &MetadataCandidate,
    duration_tolerance_seconds: u64,
) -> f32 {
    let mut score: f32 = 0.0;

    let identity_title = normalize(&identity.title);
    let identity_artist = normalize(&identity.artist);
    let identity_album = identity.album.as_ref().map(|value| normalize(value));

    let candidate_title = normalize(&candidate.title);
    let candidate_artist = normalize(&candidate.artist);
    let candidate_album = candidate.album.as_ref().map(|value| normalize(value));

    if identity_title == candidate_title {
        score += 35.0;
    } else if fuzzy_contains(&identity_title, &candidate_title) {
        score += 22.0;
    }

    if identity_artist == candidate_artist {
        score += 35.0;
    } else if fuzzy_contains(&identity_artist, &candidate_artist) {
        score += 22.0;
    }

    if let (Some(identity_album), Some(candidate_album)) = (&identity_album, &candidate_album) {
        if identity_album == candidate_album {
            score += 20.0;
        } else if fuzzy_contains(identity_album, candidate_album) {
            score += 10.0;
        }
    }

    if let (Some(identity_duration), Some(candidate_duration)) =
        (identity.duration_ms, candidate.duration_ms)
    {
        let tolerance_ms = duration_tolerance_seconds * 1000;
        let delta = identity_duration.abs_diff(candidate_duration);

        if delta <= tolerance_ms {
            score += 10.0;
        } else if delta <= tolerance_ms * 2 {
            score += 5.0;
        }
    }

    score.min(100.0_f32)
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

fn fuzzy_contains(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }

    a.contains(b) || b.contains(a)
}
