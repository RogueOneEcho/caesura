use red_oxide::jobs::JobError;
use std::path::PathBuf;

use red_oxide::logging::{Debug, Logger};
use red_oxide::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};
use red_oxide::source::SourceProvider;
use red_oxide::spectrogram::*;
use red_oxide::testing::*;

#[tokio::test]
async fn spectrogram_generator() -> Result<(), JobError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = create_shared_options(SharedOptions {
        verbosity: Some(Debug),
        output: Some(create_temp_dir("red_oxide")),
        ..SharedOptions::default()
    });
    let output_dir = shared_options.output.clone().expect("Should have value");
    let host = create_host(
        shared_options.clone(),
        SpectrogramOptions::default(),
        TranscodeOptions::default(),
    );
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
    let generated_files: Vec<PathBuf> =
        read_dir_recursive(&output_dir).expect("Should be able to read dir");
    let expected_file_count = source_file_count(&source) * 2;
    assert_eq!(generated_files.len(), expected_file_count);
    Ok(())
}
