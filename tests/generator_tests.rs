use rogue_oxide::errors::AppError;
use std::path::PathBuf;

use rogue_oxide::fs::DirectoryReader;

use rogue_oxide::logging::{Debug, Logger};
use rogue_oxide::options::SharedOptions;
use rogue_oxide::source::SourceProvider;
use rogue_oxide::spectrogram::*;
use rogue_oxide::testing::*;

#[tokio::test]
async fn spectrogram_generator() -> Result<(), AppError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = TestOptionsFactory::shared(SharedOptions {
        verbosity: Some(Debug),
        output: Some(TempDirectory::create("red_oxide")),
        ..SharedOptions::default()
    });
    let output_dir = shared_options.output.clone().expect("Should have value");
    let host = TestHostBuilder::new()
        .with_shared(shared_options.clone())
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();
    let generator = host.services.get_required::<SpectrogramGenerator>();
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_by_string(&shared_options.source.unwrap_or_default())
        .await
        .expect("Source provider should not fail");

    // Act
    generator.execute(&source).await?;

    // Assert
    let generated_files: Vec<PathBuf> = DirectoryReader::new()
        .read(&output_dir)
        .expect("Should be able to read dir");
    let expected_file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len()
        * 2;
    assert_eq!(generated_files.len(), expected_file_count);
    Ok(())
}
