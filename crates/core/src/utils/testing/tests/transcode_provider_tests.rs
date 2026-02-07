use crate::testing_prelude::*;
use std::fs;

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_provider_320() {
    // Arrange
    init_logger();

    // Act
    let config = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::_320).await;

    // Assert
    assert!(
        config.transcode_dir().exists(),
        "Transcode directory should exist: {:?}",
        config.transcode_dir()
    );
    assert!(
        config.torrent_path().exists(),
        "Torrent file should exist: {:?}",
        config.torrent_path()
    );

    // Verify MP3 files were created
    let entries: Vec<_> = fs::read_dir(config.transcode_dir())
        .expect("should read transcode dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mp3"))
        .collect();
    assert!(
        !entries.is_empty(),
        "Should have MP3 files in transcode directory"
    );
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_provider_v0() {
    // Arrange
    init_logger();

    // Act
    let config = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::V0).await;

    // Assert
    assert!(
        config.transcode_dir().exists(),
        "Transcode directory should exist: {:?}",
        config.transcode_dir()
    );
    assert!(
        config.torrent_path().exists(),
        "Torrent file should exist: {:?}",
        config.torrent_path()
    );

    // Verify MP3 files were created
    let entries: Vec<_> = fs::read_dir(config.transcode_dir())
        .expect("should read transcode dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mp3"))
        .collect();
    assert!(
        !entries.is_empty(),
        "Should have MP3 files in transcode directory"
    );
}

#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn transcode_provider_caching() {
    // Arrange
    init_logger();

    // Act - Call twice to verify caching works
    let config1 = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::_320).await;
    let config2 = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::_320).await;

    // Assert - Both should return the same paths
    assert_eq!(config1.transcode_dir(), config2.transcode_dir());
    assert_eq!(config1.torrent_path(), config2.torrent_path());
}
