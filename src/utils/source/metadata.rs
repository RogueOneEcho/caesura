use html_escape::decode_html_entities;

use crate::utils::*;
use gazelle_api::{Group, Torrent};
use log::{debug, warn};

const MAX_ARTISTS: usize = 2;
const UNKNOWN_ARTIST: &str = "Unknown Artist";
const VARIOUS_ARTISTS: &str = "Various Artists";

#[derive(Clone, Debug)]
pub struct Metadata {
    pub artist: String,
    pub album: String,
    pub remaster_title: String,
    pub year: u16,
    pub media: String,
}

impl Metadata {
    #[must_use]
    pub fn new(group: &Group, torrent: &Torrent) -> Self {
        Metadata {
            artist: get_artist(group),
            album: get_album(group),
            remaster_title: get_remaster_title(torrent),
            year: get_year(group, torrent),
            media: torrent.media.clone(),
        }
    }
}

fn get_artist(group: &Group) -> String {
    let Some(info) = group.music_info.clone() else {
        warn!("Unable to determine a suitable artist for name. Defaulting to `{UNKNOWN_ARTIST}` which likely isn't ideal");
        return UNKNOWN_ARTIST.to_owned();
    };
    let artists = if !info.artists.is_empty() && info.artists.len() <= MAX_ARTISTS {
        info.artists
    } else if info.dj.len() == 1 {
        debug!(
            "Source has {} artists so using DJ in name",
            info.artists.len()
        );
        info.dj
    } else if info.artists.is_empty() {
        warn!("Unable to determine a suitable artist for name. Defaulting to `{UNKNOWN_ARTIST}` which likely isn't ideal");
        return UNKNOWN_ARTIST.to_owned();
    } else {
        debug!(
            "Source has {} artists so name will be condensed as `{VARIOUS_ARTISTS}`",
            info.artists.len()
        );
        return VARIOUS_ARTISTS.to_owned();
    };
    let artists: Vec<String> = artists
        .into_iter()
        .map(|x| decode_html_entities(&x.name).to_string())
        .collect();
    join_humanized(artists)
}

fn get_album(group: &Group) -> String {
    decode_html_entities(&group.name).to_string()
}

fn get_remaster_title(torrent: &Torrent) -> String {
    decode_html_entities(&torrent.remaster_title).to_string()
}

fn get_year(group: &Group, torrent: &Torrent) -> u16 {
    if torrent.remaster_year.is_none() || torrent.remaster_year == Some(0) {
        group.year
    } else {
        torrent.remaster_year.expect("Remaster year should be set")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use gazelle_api::{Credit, Credits, Group};

    #[test]
    fn get_artist_none() {
        // Arrange
        let group = Group {
            music_info: Some(Credits {
                artists: Vec::new(),
                ..Credits::default()
            }),
            ..Group::default()
        };

        // Act
        let artist = get_artist(&group);

        // Assert
        assert_eq!(artist, UNKNOWN_ARTIST);
    }

    #[test]
    fn get_artist_one() {
        // Arrange
        let expected = "Hello, world!".to_owned();
        let group = Group {
            music_info: Some(Credits {
                artists: vec![Credit {
                    id: 12345,
                    name: expected.clone(),
                }],
                ..Credits::default()
            }),
            ..Group::default()
        };

        // Act
        let artist = get_artist(&group);

        // Assert
        assert_eq!(artist, expected);
    }

    #[test]
    fn get_artist_two() {
        // Arrange
        let expected = "Artist One & Artist Two".to_owned();
        let group = Group {
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 12345,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 12345,
                        name: "Artist Two".to_owned(),
                    },
                ],
                ..Credits::default()
            }),
            ..Group::default()
        };

        // Act
        let artist = get_artist(&group);

        // Assert
        assert_eq!(artist, expected);
    }

    #[test]
    fn get_artist_three() {
        // Arrange
        let expected = VARIOUS_ARTISTS.to_owned();
        let group = Group {
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 12345,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 12345,
                        name: "Artist Two".to_owned(),
                    },
                    Credit {
                        id: 12345,
                        name: "Artist Three".to_owned(),
                    },
                ],
                ..Credits::default()
            }),
            ..Group::default()
        };

        // Act
        let artist = get_artist(&group);

        // Assert
        assert_eq!(artist, expected);
    }

    #[test]
    fn get_artist_dj() {
        // Arrange
        let expected = "DJ One".to_owned();
        let group = Group {
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 12345,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 12345,
                        name: "Artist Two".to_owned(),
                    },
                    Credit {
                        id: 12345,
                        name: "Artist Three".to_owned(),
                    },
                ],
                dj: vec![Credit {
                    id: 12345,
                    name: "DJ One".to_owned(),
                }],
                ..Credits::default()
            }),
            ..Group::default()
        };

        // Act
        let artist = get_artist(&group);

        // Assert
        assert_eq!(artist, expected);
    }
}
