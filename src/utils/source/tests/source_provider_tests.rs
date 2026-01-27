use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use rogue_logging::Error;

#[tokio::test]
async fn source_provider_mocked() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let output_dir = TempDirectory::for_current_test();
    let host = HostBuilder::new()
        .with_mock_samples(SampleFormat::FLAC16_441, output_dir)
        .await
        .with_options(TargetOptions {
            allow_existing: Some(true),
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let source = provider.get(SampleDataBuilder::TORRENT_ID).await;

    // Assert
    assert!(source.is_ok());
    let source = source.expect("should have source");
    assert_eq!(source.group.name, "Test Album");
    let file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len();
    assert_eq!(file_count, 2);
    Ok(())
}
