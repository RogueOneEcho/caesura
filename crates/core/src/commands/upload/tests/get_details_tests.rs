use crate::testing_prelude::*;
use insta::assert_snapshot;

/// Test that `get_details` returns FLAC metadata for a source FLAC directory.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn get_details_flac() -> Result<(), TestError> {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let path = SAMPLE_SOURCES_DIR.join(album.dir_name());

    // Act
    let output = get_details(&path, TargetFormat::Flac).await?;

    // Assert
    assert_snapshot!(output);
    Ok(())
}

/// Test that `get_details` returns MP3 metadata for a 320kbps transcode.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn get_details_320() -> Result<(), TestError> {
    // Arrange
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let path = transcode.transcode_dir();

    // Act
    let output = get_details(&path, TargetFormat::_320).await?;

    // Assert
    assert_snapshot!(output);
    Ok(())
}

/// Test that `get_details` returns MP3 metadata for a V0 transcode.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn get_details_v0() -> Result<(), TestError> {
    // Arrange
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::V0).await;
    let path = transcode.transcode_dir();

    // Act
    let output = get_details(&path, TargetFormat::V0).await?;

    // Assert
    assert_snapshot!(output);
    Ok(())
}
