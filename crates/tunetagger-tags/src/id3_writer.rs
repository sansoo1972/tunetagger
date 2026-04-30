use id3::TagLike;
use std::path::Path;
use tunetagger_core::{ExistingTags, Lyrics, ResolvedTags, TuneTaggerError, TuneTaggerResult};

pub struct Id3Mp3TagWriter;

impl Id3Mp3TagWriter {
    pub fn read_tags(path: impl AsRef<Path>) -> TuneTaggerResult<ExistingTags> {
        let tag = id3::Tag::read_from_path(path.as_ref()).unwrap_or_default();

        let has_artwork = tag.pictures().next().is_some();
        let has_lyrics = tag.lyrics().next().is_some();

        Ok(ExistingTags {
            title: tag.title().map(ToOwned::to_owned),
            artist: tag.artist().map(ToOwned::to_owned),
            album: tag.album().map(ToOwned::to_owned),
            album_artist: tag.album_artist().map(ToOwned::to_owned),
            composer: tag
                .get("TCOM")
                .and_then(|frame| frame.content().text())
                .map(ToOwned::to_owned),
            genre: tag.genre().map(ToOwned::to_owned),
            release_date: tag.year().map(|year| year.to_string()),
            track_number: tag.track().and_then(|value| u16::try_from(value).ok()),
            disc_number: tag.disc().and_then(|value| u16::try_from(value).ok()),
            has_artwork,
            has_lyrics,
        })
    }

    pub fn write_tags(path: impl AsRef<Path>, tags: &ResolvedTags) -> TuneTaggerResult<()> {
        let path = path.as_ref();
        let mut tag = id3::Tag::read_from_path(path).unwrap_or_default();

        tag.set_title(&tags.title);
        tag.set_artist(&tags.artist);

        if let Some(value) = &tags.album {
            tag.set_album(value);
        }
        if let Some(value) = &tags.album_artist {
            tag.set_album_artist(value);
        }
        if let Some(value) = tags.track_number {
            tag.set_track(u32::from(value));
        }
        if let Some(value) = tags.track_total {
            tag.set_total_tracks(u32::from(value));
        }
        if let Some(value) = tags.disc_number {
            tag.set_disc(u32::from(value));
        }
        if let Some(value) = tags.disc_total {
            tag.set_total_discs(u32::from(value));
        }
        if let Some(value) = &tags.genre {
            tag.set_genre(value);
        }
        if let Some(value) = &tags.release_date {
            if let Ok(year) = value.get(0..4).unwrap_or(value).parse::<i32>() {
                tag.set_year(year);
            }
        }
        if let Some(value) = &tags.composer {
            tag.set_text("TCOM", value);
        }
        if let Some(value) = &tags.sort_artist {
            tag.set_text("TSOP", value);
        }
        if let Some(value) = &tags.sort_album {
            tag.set_text("TSOA", value);
        }
        if let Some(value) = &tags.sort_album_artist {
            tag.set_text("TSO2", value);
        }
        if let Some(lyrics) = &tags.lyrics {
            set_lyrics(&mut tag, lyrics);
        }

        tag.write_to_path(path, id3::Version::Id3v24)
            .map_err(|err| TuneTaggerError::Tagging(err.to_string()))
    }

    pub fn embed_artwork(
        path: impl AsRef<Path>,
        image: &[u8],
        mime_type: &str,
    ) -> TuneTaggerResult<()> {
        let path = path.as_ref();
        let mut tag = id3::Tag::read_from_path(path).unwrap_or_default();

        tag.remove("APIC");

        tag.add_frame(id3::frame::Picture {
            mime_type: normalize_mime_type(mime_type).to_string(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: String::new(),
            data: image.to_vec(),
        });

        tag.write_to_path(path, id3::Version::Id3v24)
            .map_err(|err| TuneTaggerError::Tagging(err.to_string()))
    }
}

fn set_lyrics(tag: &mut id3::Tag, lyrics: &Lyrics) {
    tag.add_frame(id3::frame::Lyrics {
        lang: lyrics.language.clone(),
        description: lyrics.description.clone().unwrap_or_default(),
        text: lyrics.text.clone(),
    });
}

fn normalize_mime_type(mime_type: &str) -> &str {
    let mime_type = mime_type.split(';').next().unwrap_or(mime_type).trim();

    match mime_type {
        "image/jpg" => "image/jpeg",
        "image/jpeg" => "image/jpeg",
        "image/png" => "image/png",
        _ => "image/jpeg",
    }
}
