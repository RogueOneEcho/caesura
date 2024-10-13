use caesura::errors::AppError;
use caesura::fs::DirectoryReader;
use caesura::hosting::HostBuilder;
use caesura::logging::{Debug, Logger};
use caesura::options::{SharedOptions, TargetOptions};
use caesura::source::*;
use caesura::testing::*;

#[tokio::test]
async fn source_provider() -> Result<(), AppError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = TestOptionsFactory::from(SharedOptions {
        verbosity: Some(Debug),
        ..SharedOptions::default()
    });
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        ..TargetOptions::default()
    });
    let host = HostBuilder::new()
        .with_options(shared_options.clone())
        .with_options(target_options)
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();

    // Act
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_from_options()
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
