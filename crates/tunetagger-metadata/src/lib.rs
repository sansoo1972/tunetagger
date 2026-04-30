pub mod apple;
pub mod musicbrainz;
pub mod scoring;

pub use apple::AppleMetadataClient;
pub use musicbrainz::{infer_album_artist_only_if_safe, MusicBrainzClient, MusicBrainzEnrichment};
