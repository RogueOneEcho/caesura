use red_oxide::formats::TargetFormatProvider;
use red_oxide::fs::DirectoryReader;
use red_oxide::jobs::JobError;
use red_oxide::logging::{Debug, Logger};
use red_oxide::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};
use red_oxide::source::SourceProvider;
use red_oxide::testing::*;
use red_oxide::transcode::SourceTranscoder;

#[tokio::test]
async fn source_transcoder() -> Result<(), JobError> {
    // Arrange
    Logger::init_new(Debug);
    let shared_options = create_shared_options(SharedOptions {
        verbosity: Some(Debug),
        output: Some(create_temp_dir("red_oxide")),
        ..SharedOptions::default()
    });
    let transcode_options = create_transcode_options(TranscodeOptions {
        allow_existing: Some(true),
        ..TranscodeOptions::default()
    });
    let output_dir = shared_options
        .output
        .clone()
        .expect("Options should be set");
    let host = create_host(
        shared_options.clone(),
        SpectrogramOptions::default(),
        transcode_options,
    );
    let provider = host.services.get_required_mut::<SourceProvider>();
    let transcoder = host.services.get_required::<SourceTranscoder>();
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_by_string(&shared_options.source.unwrap_or_default())
        .await
        .expect("Source provider should not fail");

    // Act
    transcoder.execute(&source).await?;

    // Assert
    let generated_files = DirectoryReader::new()
        .with_extensions(vec!["flac", "mp3"])
        .read(&output_dir)
        .expect("Should be able to read dir");
    let targets = host.services.get_required::<TargetFormatProvider>();
    let target_count = targets.get(source.format, &source.existing).len();
    let expected_file_count = source_file_count(&source) * target_count;
    assert_eq!(generated_files.len(), expected_file_count);
    Ok(())
}
