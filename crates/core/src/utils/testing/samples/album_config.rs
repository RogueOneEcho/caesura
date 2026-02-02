use crate::utils::{SAMPLE_SOURCES_DIR, SampleFormat, TempDirectory};
use gazelle_api::{
    Credit, Credits, Group, GroupResponse, MockGazelleClient, Torrent, TorrentResponse,
    UploadResponse,
};
use std::fs;

/// Configuration for generating a test album.
#[derive(Debug, Clone)]
pub struct AlbumConfig {
    /// Track configurations for the album.
    pub tracks: Vec<TrackConfig>,
    /// Artist name.
    pub artist: &'static str,
    /// Album title.
    pub album: &'static str,
    /// Release year.
    pub year: u16,
    /// Audio format (bit depth and sample rate).
    pub format: SampleFormat,
}

/// Configuration for a single track within an album.
#[derive(Debug, Clone)]
pub struct TrackConfig {
    /// Track title.
    pub title: &'static str,
    /// Track number (numeric like "1" or vinyl-style like "A1").
    pub track_number: &'static str,
    /// Disc number for multi-disc albums.
    pub disc_number: Option<&'static str>,
    /// Sine wave frequency in Hz for the generated audio.
    pub frequency: u32,
    /// Duration in seconds (default: 65 if None).
    pub duration_secs: Option<u32>,
}

impl Default for AlbumConfig {
    fn default() -> Self {
        Self {
            artist: "Test Artist",
            album: "Test Album",
            year: 2020,
            format: SampleFormat::default(),
            tracks: vec![
                TrackConfig {
                    title: "Track One",
                    track_number: "1",
                    disc_number: None,
                    frequency: 440,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Track Two",
                    track_number: "2",
                    disc_number: None,
                    frequency: 880,
                    duration_secs: None,
                },
            ],
        }
    }
}

impl AlbumConfig {
    /// Mock torrent ID used in tests.
    pub const TORRENT_ID: u32 = 12345;

    /// Create config with specific format, using default metadata.
    #[must_use]
    pub fn with_format(format: SampleFormat) -> Self {
        Self {
            format,
            ..Self::default()
        }
    }

    /// Create a single-disc album configuration for rename tests.
    pub fn single_disc() -> Self {
        Self {
            artist: "Rename Artist",
            album: "Single Disc Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: vec![
                TrackConfig {
                    title: "Track One",
                    track_number: "1",
                    disc_number: None,
                    frequency: 440,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Track Two",
                    track_number: "2",
                    disc_number: None,
                    frequency: 880,
                    duration_secs: None,
                },
            ],
        }
    }

    /// Create a multi-disc album configuration for rename tests.
    pub fn multi_disc() -> Self {
        Self {
            artist: "Rename Artist",
            album: "Multi Disc Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: vec![
                TrackConfig {
                    title: "First Track",
                    track_number: "1",
                    disc_number: Some("1"),
                    frequency: 440,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Second Track",
                    track_number: "2",
                    disc_number: Some("1"),
                    frequency: 550,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Third Track",
                    track_number: "1",
                    disc_number: Some("2"),
                    frequency: 660,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Fourth Track",
                    track_number: "2",
                    disc_number: Some("2"),
                    frequency: 770,
                    duration_secs: None,
                },
            ],
        }
    }

    /// Create an album with 10 tracks for testing zero-padded track numbers.
    pub fn double_digit_tracks() -> Self {
        Self {
            artist: "Rename Artist",
            album: "Double Digit Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: (1..=10_u32)
                .map(|i| {
                    let title = match i {
                        1 => "Track One",
                        2 => "Track Two",
                        3 => "Track Three",
                        4 => "Track Four",
                        5 => "Track Five",
                        6 => "Track Six",
                        7 => "Track Seven",
                        8 => "Track Eight",
                        9 => "Track Nine",
                        10 => "Track Ten",
                        _ => unreachable!(),
                    };
                    TrackConfig {
                        title,
                        track_number: Box::leak(i.to_string().into_boxed_str()),
                        disc_number: None,
                        frequency: 440 + i * 50,
                        duration_secs: None,
                    }
                })
                .collect(),
        }
    }

    /// Create an album with vinyl-style track numbers (A1, A2, B1, B2).
    pub fn vinyl_tracks() -> Self {
        Self {
            artist: "Rename Artist",
            album: "Vinyl Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: vec![
                TrackConfig {
                    title: "Side A Track One",
                    track_number: "A1",
                    disc_number: None,
                    frequency: 440,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Side A Track Two",
                    track_number: "A2",
                    disc_number: None,
                    frequency: 550,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Side B Track One",
                    track_number: "B1",
                    disc_number: None,
                    frequency: 660,
                    duration_secs: None,
                },
                TrackConfig {
                    title: "Side B Track Two",
                    track_number: "B2",
                    disc_number: None,
                    frequency: 770,
                    duration_secs: None,
                },
            ],
        }
    }

    /// Create an album with a 30-second track for testing zoom spectrogram behavior
    /// on tracks shorter than the standard 60-second start position.
    pub fn track_30s() -> Self {
        Self {
            artist: "Short Artist",
            album: "Short Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: vec![TrackConfig {
                title: "Short Track",
                track_number: "1",
                disc_number: None,
                frequency: 440,
                duration_secs: Some(30),
            }],
        }
    }

    /// Create an album with a 1-second track for testing edge case
    /// where track is shorter than the 2-second zoom capture window.
    pub fn track_1s() -> Self {
        Self {
            artist: "Very Short Artist",
            album: "Very Short Album",
            year: 2024,
            format: SampleFormat::default(),
            tracks: vec![TrackConfig {
                title: "Very Short Track",
                track_number: "1",
                disc_number: None,
                frequency: 440,
                duration_secs: Some(1),
            }],
        }
    }

    /// Directory name in standard format: `Artist - Album (Year) [WEB] {16-44.1} (FLAC)`
    pub fn dir_name(&self) -> String {
        format!(
            "{} - {} ({}) [WEB] {} (FLAC)",
            self.artist,
            self.album,
            self.year,
            self.format.dir_suffix()
        )
    }

    /// Create a temp directory containing only this album's torrent file.
    ///
    /// - Returns [`TempDirectory`] to ensure cleanup when the caller drops it
    #[must_use]
    pub fn single_torrent_dir(&self) -> TempDirectory {
        let dir = TempDirectory::create("single_torrent");
        let dest = dir.join(self.torrent_filename());
        let src = SAMPLE_SOURCES_DIR.join(self.torrent_filename());
        fs::copy(src, &dest).expect("should copy torrent file");
        dir
    }

    /// Torrent filename for this sample set.
    #[must_use]
    pub fn torrent_filename(&self) -> String {
        format!("{}.torrent", self.dir_name())
    }

    /// Track filename in format: `Artist - Album - 01 Title.flac`
    #[must_use]
    pub fn track_filename(&self, track: &TrackConfig) -> String {
        format!(
            "{} - {} - {:02} {}.flac",
            self.artist,
            self.album,
            track.track_number.parse::<u8>().unwrap_or(0),
            track.title
        )
    }

    /// File list in Gazelle API format.
    fn file_list(&self) -> String {
        self.tracks
            .iter()
            .map(|t| format!("{}{{{{{{8972941}}}}}}", self.track_filename(t)))
            .collect::<Vec<_>>()
            .join("|||")
            + "|||"
    }

    /// Build a mock API client configured for this album.
    ///
    /// - Reads the generated torrent file
    /// - Panics if torrent doesn't exist (call [`AlbumProvider::get`] first)
    #[must_use]
    pub fn api(&self) -> MockGazelleClient {
        let torrent_path = SAMPLE_SOURCES_DIR.join(self.torrent_filename());
        let torrent_bytes = fs::read(torrent_path)
            .expect("torrent file should exist - ensure AlbumProvider::get() was called first");
        build_mock_client(self, torrent_bytes, Self::TORRENT_ID, 123)
    }
}

fn build_mock_client(
    config: &AlbumConfig,
    torrent_bytes: Vec<u8>,
    torrent_id: u32,
    group_id: u32,
) -> MockGazelleClient {
    let torrent = Torrent {
        id: torrent_id,
        format: "FLAC".to_owned(),
        encoding: "Lossless".to_owned(),
        media: "WEB".to_owned(),
        remastered: true,
        remaster_year: Some(config.year),
        file_path: config.dir_name(),
        file_list: config.file_list(),
        file_count: u32::try_from(config.tracks.len()).expect("track count fits in u32"),
        ..Torrent::default()
    };

    let group = Group {
        id: group_id,
        name: config.album.to_owned(),
        year: config.year,
        category_name: "Music".to_owned(),
        music_info: Some(Credits {
            artists: vec![Credit {
                id: 1,
                name: config.artist.to_owned(),
            }],
            ..Credits::default()
        }),
        ..Group::default()
    };

    MockGazelleClient::new()
        .with_get_torrent(Ok(TorrentResponse {
            group: group.clone(),
            torrent: torrent.clone(),
        }))
        .with_get_torrent_group(Ok(GroupResponse {
            group,
            torrents: vec![torrent],
        }))
        .with_download_torrent(Ok(torrent_bytes))
        .with_upload_torrent(Ok(UploadResponse {
            torrent_id: Some(99999),
            group_id: Some(group_id),
            private: true,
            request_id: None,
            source: false,
        }))
}
