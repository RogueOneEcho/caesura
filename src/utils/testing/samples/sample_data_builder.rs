use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
use gazelle_api::{Group, GroupResponse, MockGazelleClient, Torrent, TorrentResponse};

use super::{FlacGenerator, ImageGenerator, SampleError, SampleFormat};
use crate::dependencies::ImdlCommand;
use crate::utils::SAMPLES_CONTENT_DIR;

/// Track metadata for sample generation.
pub struct TrackData {
    /// Track title.
    pub title: &'static str,
    /// Track number as a string.
    pub track_number: &'static str,
    /// Sine wave frequency in Hz for audio generation.
    pub frequency: u32,
}

/// Build sample FLAC files, cover images, torrents, and mock API clients for testing.
pub struct SampleDataBuilder {
    format: SampleFormat,
    artist: &'static str,
    album: &'static str,
    year: u16,
    tracks: Vec<TrackData>,
    directory: Option<PathBuf>,
}

impl SampleDataBuilder {
    /// Mock torrent ID used in tests.
    pub const TORRENT_ID: u32 = 12345;

    /// Create builder with the specified audio format.
    #[must_use]
    pub fn new(format: SampleFormat) -> Self {
        Self {
            format,
            artist: "Test Artist",
            album: "Test Album",
            year: 2020,
            tracks: vec![
                TrackData {
                    title: "Track One",
                    track_number: "1",
                    frequency: 440,
                },
                TrackData {
                    title: "Track Two",
                    track_number: "2",
                    frequency: 880,
                },
            ],
            directory: None,
        }
    }

    /// Set a custom output directory (default: `samples/content/{dir_name}`).
    #[must_use]
    pub fn with_directory(mut self, dir: impl AsRef<Path>) -> Self {
        self.directory = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Directory name for this sample set.
    #[must_use]
    pub fn dir_name(&self) -> String {
        format!(
            "{} - {} ({}) [WEB] {} (FLAC)",
            self.artist,
            self.album,
            self.year,
            self.format.dir_suffix()
        )
    }

    /// Build filename for a track: "Artist - Album - NN Title.flac"
    fn track_filename(&self, track: &TrackData) -> String {
        format!(
            "{} - {} - {:02} {}.flac",
            self.artist,
            self.album,
            track.track_number.parse::<u8>().unwrap_or(0),
            track.title
        )
    }

    /// Full path to the default samples directory.
    #[must_use]
    pub fn samples_dir(&self) -> String {
        format!("{}/{}", SAMPLES_CONTENT_DIR, self.dir_name())
    }

    /// Get the output directory (custom or default samples dir).
    fn output_dir(&self) -> PathBuf {
        self.directory
            .clone()
            .unwrap_or_else(|| PathBuf::from(self.samples_dir()))
    }

    /// Torrent filename for this sample set.
    #[must_use]
    pub fn torrent_filename(&self) -> String {
        format!("{}.torrent", self.dir_name())
    }

    /// Generate `FlacGenerator`s for all tracks.
    fn flac_generators(&self) -> Vec<FlacGenerator> {
        self.tracks
            .iter()
            .map(|track| {
                FlacGenerator::new()
                    .with_filename(self.track_filename(track))
                    .with_bit_depth(self.format.depth.as_u16())
                    .with_sample_rate(self.format.rate.as_u32())
                    .with_frequency(track.frequency)
                    .with_artist(self.artist)
                    .with_album(self.album)
                    .with_title(track.title)
                    .with_track_number(track.track_number)
                    .with_date(self.year.to_string())
                    .with_cover_image()
            })
            .collect()
    }

    /// Generate sample files in the output directory.
    ///
    /// - Creates FLAC files, cover image, and torrent file
    pub async fn build(&self) -> Result<(), SampleError> {
        let dir = self.output_dir();
        fs::create_dir_all(&dir).map_err(SampleError::CreateDirectory)?;
        for generator in self.flac_generators() {
            generator.generate(&dir).await?;
        }
        Self::generate_cover(&dir)?;
        self.generate_torrent().await?;
        Ok(())
    }

    fn generate_cover(dir: &Path) -> Result<(), SampleError> {
        ImageGenerator::new()
            .with_filename("cover.png")
            .generate(dir)?;
        Ok(())
    }

    async fn generate_torrent(&self) -> Result<(), SampleError> {
        let content_dir = self.output_dir();
        let torrent_path = content_dir.join(self.torrent_filename());

        ImdlCommand::create(
            &content_dir,
            &torrent_path,
            "https://flacsfor.me/test/announce".to_owned(),
            "RED".to_owned(),
        )
        .await?;

        Ok(())
    }

    /// Create a [`MockGazelleClient`] configured for this sample data.
    ///
    /// - Reads the generated torrent file from `samples_dir()`
    #[cfg(test)]
    pub fn mock_client(&self) -> Result<MockGazelleClient, SampleError> {
        use std::{fs, path::PathBuf};

        let file_list = self
            .tracks
            .iter()
            .map(|t| format!("{}{{{{{{8972941}}}}}}", self.track_filename(t)))
            .collect::<Vec<_>>()
            .join("|||")
            + "|||";

        let torrent = Torrent {
            id: Self::TORRENT_ID,
            format: "FLAC".to_owned(),
            encoding: "Lossless".to_owned(),
            media: "WEB".to_owned(),
            remastered: true,
            remaster_year: Some(self.year),
            file_path: self.dir_name(),
            file_list,
            file_count: u32::try_from(self.tracks.len()).expect("track count fits in u32"),
            ..Torrent::default()
        };

        let group = Group {
            id: 123,
            name: self.album.to_owned(),
            year: self.year,
            category_name: "Music".to_owned(),
            ..Group::default()
        };

        // Read the generated torrent file
        let torrent_path = PathBuf::from(self.samples_dir()).join(self.torrent_filename());
        let torrent_bytes = fs::read(&torrent_path).map_err(SampleError::ReadTorrent)?;

        Ok(MockGazelleClient::new()
            .with_get_torrent(Ok(TorrentResponse {
                group: group.clone(),
                torrent: torrent.clone(),
            }))
            .with_get_torrent_group(Ok(GroupResponse {
                group,
                torrents: vec![torrent],
            }))
            .with_download_torrent(Ok(torrent_bytes)))
    }
}
