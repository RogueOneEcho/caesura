//! Generate transcode outputs for testing.
//!
//! [`TranscodeGenerator`] transcodes a source album into a target format (e.g., MP3 320, V0)
//! by running the real [`TranscodeCommand`] pipeline against a mock API.
//! Caching and deduplication are handled by [`TranscodeProvider`].

use crate::testing_prelude::*;

/// Generates transcode outputs for testing.
pub(super) struct TranscodeGenerator;

impl TranscodeGenerator {
    /// Generate transcode files in the specified directory.
    ///
    /// - Writes transcoded files unconditionally
    /// - Caller is responsible for coordination (see [`TranscodeProvider`])
    pub(super) async fn generate_files(
        config: &TranscodeConfig,
        transcode_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        let output_dir = transcode_dir
            .parent()
            .expect("transcode_dir should have parent");
        tokio_create_dir_all(output_dir)
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
