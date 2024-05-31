use red_oxide::logging::{Debug, Logger};
use red_oxide::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};
use red_oxide::source::*;
use red_oxide::testing::*;

#[tokio::test]
async fn source_provider() -> Result<(), SourceError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = create_shared_options(SharedOptions {
        verbosity: Some(Debug),
        ..SharedOptions::default()
    });
    let transcode_options = create_transcode_options(TranscodeOptions {
        allow_existing: Some(true),
        ..TranscodeOptions::default()
    });
    let host = create_host(
        shared_options.clone(),
        SpectrogramOptions::default(),
        transcode_options,
    );
    let provider = host.services.get_required_mut::<SourceProvider>();

    // Act
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_by_string(&shared_options.source.unwrap_or_default())
        .await?;

    // Assert
    let file_count = source_file_count(&source);
    assert!(file_count > 0);
    Ok(())
}
