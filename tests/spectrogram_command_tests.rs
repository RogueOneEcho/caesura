use caesura::errors::AppError;
use std::path::PathBuf;

use caesura::fs::DirectoryReader;
use caesura::hosting::HostBuilder;

use caesura::logging::{Debug, Logger};
use caesura::options::SharedOptions;
use caesura::source::SourceProvider;
use caesura::spectrogram::*;
use caesura::testing::*;

#[tokio::test]
async fn spectrogram_command() -> Result<(), AppError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = TestOptionsFactory::from(SharedOptions {
        verbosity: Some(Debug),
        output: Some(TempDirectory::create("caesura")),
        ..SharedOptions::default()
    });
    let output_dir = shared_options.output.clone().expect("Should have value");
    let host = HostBuilder::new()
        .with_options(shared_options.clone())
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();
    let generator = host.services.get_required::<SpectrogramCommand>();
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_from_options()
        .await
        .expect("Source provider should not fail");

    // Act
    generator.execute().await?;

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
