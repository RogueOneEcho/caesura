//! Generate cached transcode outputs for testing.
//!
//! [`TranscodeGenerator`] uses the same file-based locking pattern as [`AlbumGenerator`]
//! to support cross-process coordination when running tests in parallel.

use std::path::Path;

use rogue_logging::Failure;
use tokio::fs::create_dir_all;

use super::lock_guard::{LockOutcome, acquire_generation_lock, mark_generated};
use super::{SampleAction, TranscodeConfig};
use crate::commands::TranscodeCommand;
use crate::hosting::HostBuilder;
use crate::options::{SharedOptions, TargetOptions};
use crate::utils::{AlbumConfig, SAMPLE_SOURCES_DIR, SourceProvider};

/// Generates cached transcode outputs for testing.
pub struct TranscodeGenerator;

impl TranscodeGenerator {
    /// Generate transcode in cached `SAMPLE_TRANSCODES_DIR` location.
    ///
    /// Uses file-based locking for cross-process coordination:
    /// - If `.generated` marker exists, skips generation
    /// - Otherwise acquires `.lock` file, generates, creates marker
    pub async fn generate(config: &TranscodeConfig) -> Result<(), Failure<SampleAction>> {
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
    ) -> Result<(), Failure<SampleAction>> {
        if let LockOutcome::Acquired(_guard) = acquire_generation_lock(transcode_dir) {
            Self::generate_files(config, transcode_dir).await?;
            mark_generated(transcode_dir);
        }
        Ok(())
    }

    /// Generate transcode files in the specified directory.
    async fn generate_files(
        config: &TranscodeConfig,
        transcode_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        let output_dir = transcode_dir
            .parent()
            .expect("transcode_dir should have parent");
        create_dir_all(output_dir)
            .await
            .map_err(Failure::wrap(SampleAction::CreateDirectory))?;
        let content_dir = SAMPLE_SOURCES_DIR.clone();
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
            .expect_build();
        let provider = host.services.get_required::<SourceProvider>();
        let transcoder = host.services.get_required::<TranscodeCommand>();
        let source = provider
            .get(AlbumConfig::TORRENT_ID)
            .await
            .map_err(Failure::wrap(SampleAction::Transcode))?
            .map_err(Failure::wrap(SampleAction::Transcode))?;
        transcoder
            .execute(&source)
            .await
            .map_err(Failure::wrap(SampleAction::Transcode))?;
        Ok(())
    }
}
