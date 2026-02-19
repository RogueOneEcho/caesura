use crate::testing_prelude::*;
use std::fs;

/// Test that `get_details` returns FLAC metadata for a multi-disc source directory.
#[tokio::test]
async fn get_details_flac() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let path = SAMPLE_SOURCES_DIR.join(config.dir_name());

    // Act
    let output = get_details(&path, false)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that `get_details` returns MP3 metadata for a 320kbps transcode.
#[tokio::test]
async fn get_details_320() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::_320).await;
    let path = transcode.transcode_dir();

    // Act
    let output = get_details(&path, false)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that `get_details` returns MP3 metadata for a V0 transcode.
#[tokio::test]
async fn get_details_v0() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::V0).await;
    let path = transcode.transcode_dir();

    // Act
    let output = get_details(&path, false)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that `get_details` handles mixed FLAC and MP3 files in the same directory.
#[tokio::test]
#[expect(
    clippy::indexing_slicing,
    reason = "test with controlled config guarantees indices exist"
)]
async fn get_details_mixed() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let flac_dir = SAMPLE_SOURCES_DIR.join(config.dir_name());
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::_320).await;
    let mp3_dir = transcode.transcode_dir();

    let temp = TempDirectory::create("mixed_format");
    fs::copy(
        flac_dir
            .join("Disc 1")
            .join(config.track_filename(&config.tracks[0])),
        temp.join(config.track_filename(&config.tracks[0])),
    )?;
    let mp3_filename = config
        .track_filename(&config.tracks[1])
        .replace(".flac", ".mp3");
    fs::copy(
        mp3_dir.join("Disc 1").join(&mp3_filename),
        temp.join(&mp3_filename),
    )?;

    // Act
    let output = get_details(&temp, false)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}
