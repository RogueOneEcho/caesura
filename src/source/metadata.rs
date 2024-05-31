use html_escape::decode_html_entities;

use crate::api::{Group, Torrent};

#[derive(Clone, Debug)]
pub struct Metadata {
    pub artist: String,
    pub album: String,
    pub remaster_title: String,
    pub year: u16,
    pub media: String,
}

impl Metadata {
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn new(group: &Group, torrent: &Torrent) -> Self {
        let artist = if group.music_info.artists.len() > 1 {
            "Various Artists".to_owned()
        } else {
            decode_html_entities(
                &group
                    .music_info
                    .artists
                    .first()
                    .expect("should be at least one artist")
                    .name,
            )
            .to_string()
        };

        let album = decode_html_entities(&group.name).to_string();
        let remaster_title = decode_html_entities(&torrent.remaster_title).to_string();
        let year = if torrent.remaster_year.is_none() || torrent.remaster_year == Some(0) {
            group.year as u16
        } else {
            torrent.remaster_year.expect("Options should be set") as u16
        };
        let media = torrent.media.clone();
        Metadata {
            artist,
            album,
            remaster_title,
            year,
            media,
        }
    }
}
