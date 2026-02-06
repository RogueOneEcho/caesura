//! Generate test albums with configurable metadata and audio format.
//!
//! [`AlbumGenerator`] generates test albums as FLAC files with cover images and torrents.
//! It supports two modes:
//!
//! - **Ephemeral**: Generate in temp directories for isolated tests
//! - **Cached**: Generate in `SAMPLES_CONTENT_DIR` with file locking for shared samples
//!
//! Use [`AlbumConfig`] to configure metadata (artist, album, tracks, disc numbers,
//! vinyl-style numbering) and audio format (bit depth, sample rate).

use std::fs;
use std::path::{Path, PathBuf};

use rogue_logging::Failure;

use super::SampleAction;
use super::lock_guard::{LockOutcome, acquire_generation_lock, mark_generated};
use crate::dependencies::ImdlCommand;
use crate::utils::testing::samples::{FlacGenerator, ImageGenerator};
use crate::utils::{AlbumConfig, SAMPLE_SOURCES_DIR};

/// Generates a complete test album (FLAC files, cover image, torrent) and mock client.
pub struct AlbumGenerator;

impl AlbumGenerator {
    /// Generate album in cached `SAMPLES_CONTENT_DIR` location.
    ///
    /// Uses file-based locking for cross-process coordination:
    /// - If `.generated` marker exists, skips generation
    /// - Otherwise acquires `.lock` file, generates, creates marker
    pub async fn generate(config: &AlbumConfig) -> Result<(), Failure<SampleAction>> {
        let source_dir = SAMPLE_SOURCES_DIR.join(config.dir_name());
        Self::generate_in_dir(config, &source_dir).await
    }

    /// Generate album in a specific directory.
    ///
    /// - Uses file-based locking for cross-process coordination
    /// - Skips generation if `.generated` marker exists
    pub async fn generate_in_dir(
        config: &AlbumConfig,
        source_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        if let LockOutcome::Acquired(_guard) = acquire_generation_lock(source_dir) {
            Self::generate_files(config, source_dir).await?;
            mark_generated(source_dir);
        }
        Ok(())
    }

    /// Generate album files in the specified directory (shared implementation).
    async fn generate_files(
        config: &AlbumConfig,
        source_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        fs::create_dir_all(source_dir).map_err(Failure::wrap(SampleAction::CreateDirectory))?;
        for track in &config.tracks {
            let mut generator = FlacGenerator::new()
                .with_filename(config.track_filename(track))
                .with_bit_depth(config.format.depth.as_u16())
                .with_sample_rate(config.format.rate.as_u32())
                .with_frequency(track.frequency)
                .with_artist(config.artist)
                .with_album(config.album)
                .with_title(track.title)
                .with_track_number(track.track_number)
                .with_date(config.year.to_string())
                .with_disc_number(track.disc_number)
                .with_cover_image();
            if let Some(duration) = track.duration_secs {
                generator = generator.with_duration_secs(duration);
            }
            generator.generate(source_dir).await?;
        }
        ImageGenerator::new()
            .with_filename("cover.png")
            .generate(source_dir)?;
        let torrent_path = append_extension(source_dir, "torrent");
        ImdlCommand::create(
            source_dir,
            &torrent_path,
            "https://flacsfor.me/test/announce".to_owned(),
            "RED".to_owned(),
        )
        .await
        .map_err(Failure::wrap(SampleAction::CreateTorrent))?;
        Ok(())
    }
}

/// Append an extension to a path.
///
/// Unlike [`Path::with_extension`], this appends rather than replaces.
/// Needed for paths containing dots like `{16-44.1} (FLAC)`.
fn append_extension(path: &Path, ext: &str) -> PathBuf {
    let mut result = path.as_os_str().to_os_string();
    result.push(".");
    result.push(ext);
    PathBuf::from(result)
}
