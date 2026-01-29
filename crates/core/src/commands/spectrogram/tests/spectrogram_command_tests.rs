use insta::assert_yaml_snapshot;

use crate::commands::*;
use crate::hosting::*;
use crate::utils::*;

#[tokio::test]
async fn spectrogram_command_flac16_441() {
    let album = AlbumConfig::with_format(SampleFormat::FLAC16_441);
    let snapshot = spectrogram_command_helper(album).await;
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
async fn spectrogram_command_flac16_48() {
    let album = AlbumConfig::with_format(SampleFormat::FLAC16_48);
    let snapshot = spectrogram_command_helper(album).await;
    assert_yaml_snapshot!(snapshot);
}

/// Test zoom spectrogram for a 30-second track (shorter than the 60-second standard position).
/// Should capture at 50% mark instead.
#[tokio::test]
async fn spectrogram_command_track_30s() {
    let album = AlbumConfig::track_30s();
    let snapshot = spectrogram_command_helper(album).await;
    assert_yaml_snapshot!(snapshot);
}

/// Test zoom spectrogram for a 1-second track (shorter than the 2-second capture window).
/// Sox should gracefully capture whatever audio exists.
#[tokio::test]
async fn spectrogram_command_track_1s() {
    let album = AlbumConfig::track_1s();
    let snapshot = spectrogram_command_helper(album).await;
    assert_yaml_snapshot!(snapshot);
}

async fn spectrogram_command_helper(album: AlbumConfig) -> Vec<FileSnapshot> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    AlbumGenerator::generate(&album)
        .await
        .expect("should generate album");
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let generator = host.services.get_required::<SpectrogramCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // Act
    let status = generator.execute(&source).await;

    // Assert
    assert!(status.success);
    DirectorySnapshot::new()
        .with_directory(test_dir.output())
        .create()
        .expect("should read output directory")
}
