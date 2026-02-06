use crate::testing_prelude::*;

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_command_flac16_441() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC16_441).await;
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_command_flac16_48() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC16_48).await;
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_command_flac24_441() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_441).await;
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_command_flac24_48() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_48).await;
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_command_flac24_96() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_96).await;
    assert_yaml_snapshot!(snapshot);
}

async fn transcode_command_helper(format: SampleFormat) -> Vec<FileSnapshot> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(format).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");

    // Act
    let result = transcoder.execute(&source).await;

    // Assert
    assert!(result.is_ok(), "transcode should succeed");
    DirectorySnapshot::new()
        .with_directory(test_dir.output())
        .without_extensions(&["torrent"])
        .create()
        .expect("should read output directory")
}
