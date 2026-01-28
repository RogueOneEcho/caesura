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

use std::fs::{self, File};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::SampleError;
use crate::dependencies::ImdlCommand;
use crate::utils::AlbumConfig;
use crate::utils::testing::samples::{FlacGenerator, ImageGenerator};

/// Generates a complete test album (FLAC files, cover image, torrent) and mock client.
pub struct AlbumGenerator;

const TIMEOUT_SECONDS: u64 = 10;
const POLL_MILLISECONDS: u64 = 500;

impl AlbumGenerator {
    /// Generate album in cached `SAMPLES_CONTENT_DIR` location.
    ///
    /// Uses file-based locking for cross-process coordination:
    /// - If `.generated` marker exists, skips generation
    /// - Otherwise acquires `.lock` file, generates, creates marker
    pub async fn generate(config: &AlbumConfig) -> Result<(), SampleError> {
        let source_dir = config.source_dir();
        Self::generate_in_dir(config, &source_dir).await
    }

    /// Generate album in a specific directory.
    ///
    /// - Uses file-based locking for cross-process coordination
    /// - Skips generation if `.generated` marker exists
    pub async fn generate_in_dir(
        config: &AlbumConfig,
        source_dir: &Path,
    ) -> Result<(), SampleError> {
        let marker = source_dir.join(".generated");
        let lock = source_dir.join(".lock");

        // Fast path: already generated
        if !marker.exists() {
            // Try to acquire lock
            if let Some(parent) = lock.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock)
                .is_ok()
            {
                // We got the lock - generate samples
                Self::generate_files(config, source_dir).await;
                // Create marker on success
                let _ = File::create(&marker);
                // Release lock
                let _ = fs::remove_file(&lock);
            } else {
                // Lock exists - wait for completion
                wait_for_generation(&marker, &lock);
            }
        }
        Ok(())
    }

    /// Generate album files in the specified directory (shared implementation).
    async fn generate_files(config: &AlbumConfig, source_dir: &Path) {
        fs::create_dir_all(source_dir).expect("should create source directory");

        // Generate FLAC files
        for track in &config.tracks {
            FlacGenerator::new()
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
                .with_cover_image()
                .generate(source_dir)
                .await
                .expect("should generate FLAC");
        }

        // Generate cover image
        ImageGenerator::new()
            .with_filename("cover.png")
            .generate(source_dir)
            .expect("should generate cover");

        // Generate torrent
        let torrent_path = config.torrent_path();
        ImdlCommand::create(
            source_dir,
            &torrent_path,
            "https://flacsfor.me/test/announce".to_owned(),
            "RED".to_owned(),
        )
        .await
        .expect("should create torrent");
    }
}

fn wait_for_generation(marker: &Path, lock: &Path) {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let poll = Duration::from_millis(POLL_MILLISECONDS);
    let start = Instant::now();
    while start.elapsed() < timeout {
        if marker.exists() {
            return; // Generation complete
        }
        // Lock released but no marker - generation failed
        assert!(lock.exists(), "Sample generation failed in another process");
        thread::sleep(poll);
    }
    unreachable!("Timeout waiting for sample generation");
}
