use rogue_oxide::errors::AppError;
use rogue_oxide::fs::DirectoryReader;
use rogue_oxide::logging::{Debug, Logger};
use rogue_oxide::options::{SharedOptions, TranscodeOptions};
use rogue_oxide::source::*;
use rogue_oxide::testing::*;

#[tokio::test]
async fn source_provider() -> Result<(), AppError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = TestOptionsFactory::shared(SharedOptions {
        verbosity: Some(Debug),
        ..SharedOptions::default()
    });
    let transcode_options = TestOptionsFactory::transcode(TranscodeOptions {
        allow_existing: Some(true),
        ..TranscodeOptions::default()
    });
    let host = TestHostBuilder::new()
        .with_shared(shared_options.clone())
        .with_transcode(transcode_options)
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();

    // Act
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_by_string(&shared_options.source.unwrap_or_default())
        .await?;

    // Assert
    let file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len();
    assert!(file_count > 0);
    Ok(())
}
