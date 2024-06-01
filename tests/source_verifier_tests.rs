use rogue_oxide::errors::AppError;
use rogue_oxide::logging::{Debug, Logger};
use rogue_oxide::options::{SharedOptions, TranscodeOptions};
use rogue_oxide::source::*;
use rogue_oxide::testing::*;
use rogue_oxide::verify::SourceVerifier;

#[tokio::test]
async fn source_verifier() -> Result<(), AppError> {
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
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_by_string(&shared_options.source.unwrap_or_default())
        .await?;
    let verifier = host.services.get_required_mut::<SourceVerifier>();
    let mut verifier = verifier
        .write()
        .expect("verifier should be available to write");

    // Act
    let _is_verified = verifier.execute(&source).await?;

    // Assert not required
    Ok(())
}
