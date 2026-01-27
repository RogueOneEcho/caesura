use std::fs;
use std::path::{Path, PathBuf};

use tokio::process::Command;

use super::{CommandExt, ImageGenerator, SampleError};
use crate::dependencies::{METAFLAC, SOX};

/// Builder for generating sample FLAC files.
///
/// # Example
/// ```ignore
/// // With automatic filename from metadata
/// FlacGenerator::new()
///     .with_artist("Artist Name")
///     .with_album("Album Name")
///     .with_title("Track Title")
///     .with_track_number("1")
///     .generate(&output_dir)
///     .await?;
///
/// // With explicit filename
/// FlacGenerator::new()
///     .with_filename("custom.flac")
///     .with_sample_rate(48000)
///     .generate(&output_dir)
///     .await?;
///
/// // With embedded cover image
/// FlacGenerator::new()
///     .with_cover_image()
///     .generate(&output_dir)
///     .await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct FlacGenerator {
    // Audio parameters
    sample_rate: Option<u32>,
    bit_depth: Option<u16>,
    channels: Option<u8>,
    duration_secs: Option<u32>,
    frequency: Option<u32>,

    // File location
    filename: Option<String>,
    sub_directory: Option<String>,

    // Metadata
    artist: Option<String>,
    album: Option<String>,
    title: Option<String>,
    track_number: Option<String>,
    date: Option<String>,

    // Cover image
    embed_cover: bool,
}

impl FlacGenerator {
    // Default values
    const DEFAULT_SAMPLE_RATE: u32 = 44100;
    const DEFAULT_BIT_DEPTH: u16 = 16;
    const DEFAULT_CHANNELS: u8 = 2;
    const DEFAULT_DURATION_SECS: u32 = 65;
    const DEFAULT_FREQUENCY: u32 = 440;

    /// Create a new FLAC generator with default audio parameters.
    ///
    /// Defaults:
    /// - Sample rate: 44100 Hz
    /// - Bit depth: 16
    /// - Channels: 2 (stereo)
    /// - Duration: 65 seconds
    /// - Frequency: 440 Hz (A4 note)
    /// - Filename: generated from metadata (e.g., "01 - Title.flac")
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an explicit filename (default: generated from metadata).
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set the sample rate in Hz (default: 44100).
    #[must_use]
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }

    /// Set the bit depth (default: 16).
    #[must_use]
    pub fn with_bit_depth(mut self, depth: u16) -> Self {
        self.bit_depth = Some(depth);
        self
    }

    /// Set the number of channels (default: 2).
    #[must_use]
    #[expect(dead_code)]
    pub fn with_channels(mut self, channels: u8) -> Self {
        self.channels = Some(channels);
        self
    }

    /// Set the duration in seconds (default: 65).
    #[must_use]
    #[expect(dead_code)]
    pub fn with_duration_secs(mut self, secs: u32) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    /// Set the sine wave frequency in Hz (default: 440).
    #[must_use]
    pub fn with_frequency(mut self, freq: u32) -> Self {
        self.frequency = Some(freq);
        self
    }

    /// Set a sub-directory within the output directory.
    #[must_use]
    #[expect(dead_code)]
    pub fn with_sub_directory(mut self, sub_dir: impl Into<String>) -> Self {
        self.sub_directory = Some(sub_dir.into());
        self
    }

    /// Set the ARTIST metadata tag.
    #[must_use]
    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    /// Set the ALBUM metadata tag.
    #[must_use]
    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    /// Set the TITLE metadata tag.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the TRACKNUMBER metadata tag.
    #[must_use]
    pub fn with_track_number(mut self, track: impl Into<String>) -> Self {
        self.track_number = Some(track.into());
        self
    }

    /// Set the DATE metadata tag.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Embed a cover image (100x100 gradient PNG).
    #[must_use]
    pub fn with_cover_image(mut self) -> Self {
        self.embed_cover = true;
        self
    }

    /// Build the filename from metadata.
    ///
    /// Format: "{`track_number`} - {title}.flac" or "{title}.flac" or "track.flac"
    fn build_filename(&self) -> String {
        match (&self.track_number, &self.title) {
            (Some(track), Some(title)) => format!("{track} - {title}.flac"),
            (None, Some(title)) => format!("{title}.flac"),
            (Some(track), None) => format!("{track} - track.flac"),
            (None, None) => "track.flac".to_owned(),
        }
    }

    /// Generate the FLAC file in the specified output directory.
    ///
    /// Returns the full path to the generated file.
    pub async fn generate(&self, output_dir: &Path) -> Result<PathBuf, SampleError> {
        let dir = match &self.sub_directory {
            Some(sub) => output_dir.join(sub),
            None => output_dir.to_path_buf(),
        };
        fs::create_dir_all(&dir).map_err(SampleError::CreateDirectory)?;

        let filename = self
            .filename
            .clone()
            .unwrap_or_else(|| self.build_filename());
        let path = dir.join(filename);

        let sample_rate = self.sample_rate.unwrap_or(Self::DEFAULT_SAMPLE_RATE);
        let bit_depth = self.bit_depth.unwrap_or(Self::DEFAULT_BIT_DEPTH);
        let channels = self.channels.unwrap_or(Self::DEFAULT_CHANNELS);
        let duration_secs = self.duration_secs.unwrap_or(Self::DEFAULT_DURATION_SECS);
        let frequency = self.frequency.unwrap_or(Self::DEFAULT_FREQUENCY);

        // Generate sine wave with SOX
        // -D disables dithering for deterministic output
        Command::new(SOX)
            .args([
                "-D",
                "-n",
                "-r",
                &sample_rate.to_string(),
                "-b",
                &bit_depth.to_string(),
                "-c",
                &channels.to_string(),
            ])
            .arg(&path)
            .args([
                "synth",
                &duration_secs.to_string(),
                "sine",
                &frequency.to_string(),
            ])
            .run()
            .await
            .map_err(SampleError::Sox)?;

        // Add metadata with metaflac
        self.apply_metadata(&path).await?;

        // Add cover image if configured
        if self.embed_cover {
            let image_path = ImageGenerator::new()
                .with_filename("cover_temp.png")
                .generate(&dir)?;
            self.apply_picture(&path, &image_path).await?;
            fs::remove_file(&image_path).map_err(SampleError::RemoveFile)?;
        }

        Ok(path)
    }

    async fn apply_metadata(&self, path: &Path) -> Result<(), SampleError> {
        let mut args: Vec<String> = Vec::new();

        if let Some(artist) = &self.artist {
            args.push(format!("--set-tag=ARTIST={artist}"));
        }
        if let Some(album) = &self.album {
            args.push(format!("--set-tag=ALBUM={album}"));
        }
        if let Some(title) = &self.title {
            args.push(format!("--set-tag=TITLE={title}"));
        }
        if let Some(track) = &self.track_number {
            args.push(format!("--set-tag=TRACKNUMBER={track}"));
        }
        if let Some(date) = &self.date {
            args.push(format!("--set-tag=DATE={date}"));
        }

        if args.is_empty() {
            return Ok(());
        }

        Command::new(METAFLAC)
            .args(&args)
            .arg(path)
            .run()
            .await
            .map_err(SampleError::MetaflacTags)?;

        Ok(())
    }

    async fn apply_picture(&self, flac_path: &Path, image_path: &Path) -> Result<(), SampleError> {
        // Format: [TYPE]|[MIME-TYPE]|[DESCRIPTION]|[WIDTHxHEIGHTxDEPTH[/COLORS]]|FILE
        // Type 3 = front cover
        // Leave width/height/depth empty for metaflac to auto-detect
        let spec = format!("3|image/png|||{}", image_path.display());

        Command::new(METAFLAC)
            .arg(format!("--import-picture-from={spec}"))
            .arg(flac_path)
            .run()
            .await
            .map_err(SampleError::MetaflacPicture)?;

        Ok(())
    }
}
