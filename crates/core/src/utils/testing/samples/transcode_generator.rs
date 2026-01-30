//! Generate cached transcode outputs for testing.
//!
//! [`TranscodeGenerator`] uses the same file-based locking pattern as [`AlbumGenerator`]
//! to support cross-process coordination when running tests in parallel.

use std::fs::{self, File};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::{SampleError, TranscodeConfig};
use crate::commands::TranscodeCommand;
use crate::hosting::HostBuilder;
use crate::options::{CacheOptions, SharedOptions, TargetOptions};
use crate::utils::{AlbumConfig, SourceProvider, TempDirectory};
use tokio::fs::create_dir_all;

/// Generates cached transcode outputs for testing.
pub struct TranscodeGenerator;

const TIMEOUT_SECONDS: u64 = 60;
const POLL_MILLISECONDS: u64 = 500;

impl TranscodeGenerator {
    /// Generate transcode in cached `SAMPLE_TRANSCODES_DIR` location.
    ///
    /// Uses file-based locking for cross-process coordination:
    /// - If `.generated` marker exists, skips generation
    /// - Otherwise acquires `.lock` file, generates, creates marker
    pub async fn generate(config: &TranscodeConfig) -> Result<(), SampleError> {
        let transcode_dir = config.transcode_dir();
        Self::generate_in_dir(config, &transcode_dir).await
    }

    /// Generate transcode in a specific directory.
    ///
    /// - Uses file-based locking for cross-process coordination
    /// - Skips generation if `.generated` marker exists
    pub async fn generate_in_dir(
        config: &TranscodeConfig,
        transcode_dir: &Path,
    ) -> Result<(), SampleError> {
        let marker = transcode_dir.join(".generated");
        let lock = transcode_dir.join(".lock");

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
                // We got the lock - generate transcode
                Self::generate_files(config, transcode_dir).await?;
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

    /// Generate transcode files in the specified directory.
    async fn generate_files(
        config: &TranscodeConfig,
        transcode_dir: &Path,
    ) -> Result<(), SampleError> {
        // Create output directory
        let output_dir = transcode_dir
            .parent()
            .expect("transcode_dir should have parent");
        create_dir_all(output_dir)
            .await
            .map_err(SampleError::CreateDirectory)?;

        // Create a temporary cache directory in /tmp
        let cache_dir = TempDirectory::create("transcode_cache");
        let content_dir = config
            .album
            .source_dir()
            .parent()
            .expect("has parent")
            .to_path_buf();

        // Build a minimal DI host for transcoding
        let host = HostBuilder::new()
            .with_mock_api(config.album.clone())
            .with_options(SharedOptions {
                content: vec![content_dir],
                output: output_dir.to_path_buf(),
                ..SharedOptions::mock()
            })
            .with_options(TargetOptions {
                target: vec![config.target],
                ..TargetOptions::default()
            })
            .with_options(CacheOptions { cache: cache_dir })
            .build();
        let provider = host.services.get_required::<SourceProvider>();
        let transcoder = host.services.get_required::<TranscodeCommand>();

        // Get the source
        let source = provider
            .get(AlbumConfig::TORRENT_ID)
            .await
            .map_err(|e| SampleError::Transcode(e.to_string()))?;

        // Execute the transcode
        let status = transcoder.execute(&source).await;
        if !status.success {
            let error_msg = status
                .error
                .map_or_else(|| "Unknown transcode error".to_owned(), |e| e.to_string());
            return Err(SampleError::Transcode(error_msg));
        }

        Ok(())
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
        assert!(
            lock.exists(),
            "Transcode generation failed in another process"
        );
        thread::sleep(poll);
    }
    unreachable!("Timeout waiting for transcode generation");
}
