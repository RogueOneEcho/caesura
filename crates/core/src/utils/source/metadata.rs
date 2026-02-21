use crate::prelude::*;
use gazelle_api::{Credit, Credits, Group, Torrent};
use html_escape::decode_html_entities;
use serde::Serialize;

const MAX_ARTISTS: usize = 2;
const UNKNOWN_ARTIST: &str = "Unknown Artist";
const VARIOUS_ARTISTS: &str = "Various Artists";

/// Album metadata extracted from the API.
#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    /// Artist display name derived from credits.
    ///
    /// - 1-2 main artists: joined with `&`
    /// - 3+ main artists: falls back to single DJ or composer, else `Various Artists`
    /// - No main artists: falls back to guest artists, else `Unknown Artist`
    pub artist: String,
    /// Album title.
    pub album: String,
    /// Edition title.
    pub edition_title: Option<String>,
    /// Edition year if set, else original year.
    pub year: u16,
    /// Media type.
    pub media: String,
    /// Original year.
    ///
    /// `None` if date is invalid.
    pub original_year: Option<u16>,
    /// Edition year.
    ///
    /// `None` if not an edition or date is invalid.
    pub edition_year: Option<u16>,
    /// Edition record label.
    pub record_label: Option<String>,
    /// Edition catalogue number.
    pub catalogue_number: Option<String>,
    /// Artists
    pub artists: Vec<String>,
    /// Composers.
    ///
    /// Typically present for classical works.
    pub composers: Vec<String>,
    /// Conductors.
    ///
    /// Typically present for classical works.
    pub conductor: Vec<String>,
    /// DJs
    pub dj: Vec<String>,
    /// Producers
    pub producer: Vec<String>,
    /// Remix artist
    pub remixed_by: Vec<String>,
    /// Featured artists
    pub with: Vec<String>,
    /// Arrangers.
    ///
    /// *OPS only*
    pub arranger: Vec<String>,
}

impl Metadata {
    /// Create [`Metadata`] from API response.
    #[must_use]
    pub(crate) fn new_with_logging(group: &Group, torrent: &Torrent) -> Self {
        Self::new(group, torrent, true)
    }

    /// Create [`Metadata`] from API response without logging warnings.
    #[must_use]
    fn new(group: &Group, torrent: &Torrent, log: bool) -> Self {
        Metadata {
            artist: get_artist(group, log),
            album: get_album(group),
            edition_title: get_remaster_title(torrent),
            year: get_year(group, torrent),
            media: torrent.media.clone(),
            original_year: optional_year(Some(group.year)),
            edition_year: optional_year(torrent.remaster_year),
            record_label: optional_string(torrent.remaster_record_label.clone()),
            catalogue_number: optional_string(torrent.remaster_catalogue_number.clone()),
            artists: credits(group, |a| &a.artists),
            composers: credits(group, |a| &a.composers),
            conductor: credits(group, |a| &a.conductor),
            dj: credits(group, |a| &a.dj),
            producer: credits(group, |a| &a.producer),
            remixed_by: credits(group, |a| &a.remixed_by),
            with: credits(group, |a| &a.with),
            arranger: credits(group, |a| a.arranger.as_deref().unwrap_or_default()),
        }
    }
    /// Create a [`Metadata`] with placeholder values for testing.
    #[must_use]
    pub(crate) fn mock() -> Self {
        Metadata {
            artist: "Mock Artist".to_owned(),
            album: "Test Album".to_owned(),
            edition_title: Some("Deluxe Edition".to_owned()),
            year: 2020,
            original_year: Some(2001),
            edition_year: Some(2020),
            media: "CD".to_owned(),
            record_label: Some("Mock Records".to_owned()),
            catalogue_number: Some("0123456789".to_owned()),
            artists: vec!["Mock Artist".to_owned()],
            composers: Vec::new(),
            conductor: Vec::new(),
            dj: Vec::new(),
            producer: Vec::new(),
            remixed_by: Vec::new(),
            with: Vec::new(),
            arranger: Vec::new(),
        }
    }

    /// Create a [`Metadata`] with two artists for testing.
    ///
    /// Uses two artists to trigger the joined artist name path.
    #[must_use]
    pub(crate) fn mock_2_artists() -> Self {
        let group = Group {
            name: "Test Album".to_owned(),
            year: 2020,
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 1,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 2,
                        name: "Artist Two".to_owned(),
                    },
                ],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_year: Some(2020),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] with multiple artists for testing.
    ///
    /// Uses three artists to trigger the "Various Artists" condensation path.
    #[must_use]
    pub(crate) fn mock_3_artists() -> Self {
        let group = Group {
            name: "Test Album".to_owned(),
            year: 2020,
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 1,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 2,
                        name: "Artist Two".to_owned(),
                    },
                    Credit {
                        id: 3,
                        name: "Artist Three".to_owned(),
                    },
                ],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_title: "Deluxe Edition".to_owned(),
            remaster_year: Some(2020),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] for a DJ mix release.
    ///
    /// Uses three artists and one DJ to trigger the DJ fallback path.
    #[must_use]
    pub(crate) fn mock_3_artists_1_dj() -> Self {
        let group = Group {
            name: "Test Mix".to_owned(),
            year: 2020,
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 1,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 2,
                        name: "Artist Two".to_owned(),
                    },
                    Credit {
                        id: 3,
                        name: "Artist Three".to_owned(),
                    },
                ],
                dj: vec![Credit {
                    id: 4,
                    name: "Mock DJ".to_owned(),
                }],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "WEB".to_owned(),
            remaster_year: Some(2020),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] for a classical release with a single composer.
    ///
    /// Uses three artists and one composer to trigger the composer fallback path.
    #[must_use]
    pub(crate) fn mock_3_artists_1_composer() -> Self {
        let group = Group {
            name: "Symphony No. 42 in G Minor".to_owned(),
            year: 1888,
            music_info: Some(Credits {
                artists: vec![
                    Credit {
                        id: 1,
                        name: "Artist One".to_owned(),
                    },
                    Credit {
                        id: 2,
                        name: "Artist Two".to_owned(),
                    },
                    Credit {
                        id: 3,
                        name: "Artist Three".to_owned(),
                    },
                ],
                composers: vec![Credit {
                    id: 4,
                    name: "Mock Composer".to_owned(),
                }],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_year: Some(1996),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] for a classical release with one artist and one composer.
    ///
    /// Uses one artist so the normal artist path is taken (composer is ignored).
    #[must_use]
    pub(crate) fn mock_1_artist_1_composer() -> Self {
        let group = Group {
            name: "Symphony No. 42 in G Minor".to_owned(),
            year: 1888,
            music_info: Some(Credits {
                artists: vec![Credit {
                    id: 1,
                    name: "Mock Artist".to_owned(),
                }],
                composers: vec![Credit {
                    id: 2,
                    name: "Mock Composer".to_owned(),
                }],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_year: Some(1996),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] with guest artists only.
    ///
    /// Uses no main artists and two guests to trigger the guest artist fallback path.
    #[must_use]
    pub(crate) fn mock_0_artists_2_guests() -> Self {
        let group = Group {
            name: "Test Album".to_owned(),
            year: 2020,
            music_info: Some(Credits {
                with: vec![
                    Credit {
                        id: 1,
                        name: "Guest One".to_owned(),
                    },
                    Credit {
                        id: 2,
                        name: "Guest Two".to_owned(),
                    },
                ],
                ..Credits::default()
            }),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_year: Some(2020),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }

    /// Create a [`Metadata`] with no artist information.
    ///
    /// Uses empty credits to trigger the "Unknown Artist" fallback path.
    #[must_use]
    pub(crate) fn mock_0_artists() -> Self {
        let group = Group {
            name: "Test Album".to_owned(),
            year: 2020,
            music_info: Some(Credits::default()),
            ..Group::default()
        };
        let torrent = Torrent {
            media: "CD".to_owned(),
            remaster_year: Some(2020),
            ..Torrent::default()
        };
        Self::new(&group, &torrent, false)
    }
}

fn get_artist(group: &Group, log: bool) -> String {
    let Some(info) = group.music_info.clone() else {
        if log {
            warn!(
                "Unable to determine a suitable artist for name. Defaulting to `{UNKNOWN_ARTIST}` which likely isn't ideal"
            );
        }
        return UNKNOWN_ARTIST.to_owned();
    };
    let artists = if !info.artists.is_empty() && info.artists.len() <= MAX_ARTISTS {
        info.artists
    } else if info.dj.len() == 1 {
        if log {
            debug!(
                "Source has {} main artists so using DJ in name",
                info.artists.len()
            );
        }
        info.dj
    } else if info.composers.len() == 1 {
        if log {
            debug!(
                "Source has {} main artists so using composer in name",
                info.artists.len()
            );
        }
        info.composers
    } else if info.artists.is_empty() {
        if info.with.is_empty() {
            if log {
                warn!(
                    "Source has no main or guest artists. Defaulting to `{UNKNOWN_ARTIST}` which likely isn't ideal"
                );
            }
            return UNKNOWN_ARTIST.to_owned();
        } else if info.with.len() <= MAX_ARTISTS {
            if log {
                warn!("Source has no main artist so using guest artists in name");
            }
            info.with
        } else {
            if log {
                debug!(
                    "Source has no main artist and {} guest artists so name will be condensed as `{VARIOUS_ARTISTS}`",
                    info.with.len()
                );
            }
            return VARIOUS_ARTISTS.to_owned();
        }
    } else {
        if log {
            debug!(
                "Source has {} main artists so name will be condensed as `{VARIOUS_ARTISTS}`",
                info.artists.len()
            );
        }
        return VARIOUS_ARTISTS.to_owned();
    };
    let artists: Vec<String> = artists
        .into_iter()
        .map(|x| decode_html_entities(&x.name).to_string())
        .collect();
    and_join(artists)
}

fn get_album(group: &Group) -> String {
    decode_html_entities(&group.name).to_string()
}

fn get_remaster_title(torrent: &Torrent) -> Option<String> {
    let title = decode_html_entities(&torrent.remaster_title).to_string();
    if title.is_empty() { None } else { Some(title) }
}

fn get_year(group: &Group, torrent: &Torrent) -> u16 {
    match torrent.remaster_year {
        Some(year) if year != 0 => year,
        _ => group.year,
    }
}

fn optional_year(year: Option<u16>) -> Option<u16> {
    match year {
        Some(year) if year != 0 => Some(year),
        _ => None,
    }
}

fn optional_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn credits(group: &Group, predicate: impl Fn(&Credits) -> &[Credit]) -> Vec<String> {
    let Some(credits) = group.music_info.as_ref() else {
        return Vec::new();
    };
    predicate(credits)
        .iter()
        .map(|x| decode_html_entities(&x.name).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_artist_none() {
        // Act
        let metadata = Metadata::mock_0_artists();

        // Assert
        assert_eq!(metadata.artist, UNKNOWN_ARTIST);
    }

    #[test]
    fn get_artist_one() {
        // Act
        let metadata = Metadata::mock();

        // Assert
        assert_eq!(metadata.artist, "Mock Artist");
    }

    #[test]
    fn get_artist_two() {
        // Act
        let metadata = Metadata::mock_2_artists();

        // Assert
        assert_eq!(metadata.artist, "Artist One & Artist Two");
    }

    #[test]
    fn get_artist_three() {
        // Act
        let metadata = Metadata::mock_3_artists();

        // Assert
        assert_eq!(metadata.artist, VARIOUS_ARTISTS);
    }

    #[test]
    fn get_artist_dj() {
        // Act
        let metadata = Metadata::mock_3_artists_1_dj();

        // Assert
        assert_eq!(metadata.artist, "Mock DJ");
    }

    #[test]
    fn get_artist_composer() {
        // Act
        let metadata = Metadata::mock_3_artists_1_composer();

        // Assert
        assert_eq!(metadata.artist, "Mock Composer");
    }

    #[test]
    fn get_artist_1_artist_1_composer() {
        // Act
        let metadata = Metadata::mock_1_artist_1_composer();

        // Assert
        assert_eq!(metadata.artist, "Mock Artist");
    }

    #[test]
    fn get_artist_guest_only() {
        // Act
        let metadata = Metadata::mock_0_artists_2_guests();

        // Assert
        assert_eq!(metadata.artist, "Guest One & Guest Two");
    }
}
