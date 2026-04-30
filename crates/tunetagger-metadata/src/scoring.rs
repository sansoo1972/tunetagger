use tunetagger_core::{MetadataCandidate, TrackIdentity};

pub fn score_candidate(
    identity: &TrackIdentity,
    candidate: &MetadataCandidate,
    duration_tolerance_seconds: u16,
) -> f32 {
    let mut score = 0.0;

    if eq_norm(&identity.title, &candidate.title) {
        score += 30.0;
    }

    if eq_norm(&identity.artist, &candidate.artist) {
        score += 30.0;
    }

    if let (Some(a), Some(b)) = (identity.duration_ms, candidate.duration_ms) {
        let delta = a.abs_diff(b);
        if delta <= u64::from(duration_tolerance_seconds) * 1000 {
            score += 20.0;
        }
    }

    if let (Some(a), Some(b)) = (&identity.album, &candidate.album) {
        if eq_norm(a, b) {
            score += 10.0;
        }
    }

    score
}

fn eq_norm(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}
