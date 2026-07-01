use crate::testing_prelude::*;
use std::fs::metadata as fs_metadata;

#[tokio::test]
async fn torrent_file_provider_get_repeat_call() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<TorrentFileProvider>();
    let paths = host.services.get_required::<PathManager>();
    let torrent_path = paths.get_source_torrent_path(AlbumConfig::TORRENT_ID);
    assert!(!torrent_path.is_file(), "torrent should not be cached yet");

    // Act
    let result = provider.get(AlbumConfig::TORRENT_ID).await?;

    // Assert
    assert_eq!(result, torrent_path);
    assert!(
        torrent_path.is_file(),
        "torrent should be cached after download"
    );
    let metadata = fs_metadata(&torrent_path)?;
    let first_modified = metadata.modified()?;

    // Act
    let result = provider.get(AlbumConfig::TORRENT_ID).await?;

    // Assert
    assert_eq!(result, torrent_path);
    let metadata = fs_metadata(&torrent_path)?;
    let second_modified = metadata.modified()?;
    assert_eq!(
        first_modified, second_modified,
        "cached file should not be rewritten"
    );
    Ok(())
}

/// A failed download must not leave a file at the final cached torrent path,
/// or subsequent runs will treat the empty/partial file as a valid cache entry.
#[tokio::test]
async fn torrent_file_provider_get_download_failure() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let failing_api = album.api().with_download_torrent(Err(GazelleError {
        operation: GazelleOperation::SendRequest,
        source: ErrorSource::Io(IoError::new(
            ErrorKind::ConnectionRefused,
            "simulated download failure",
        )),
    }));
    let host = HostBuilder::new()
        .with_mock_client(failing_api)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<TorrentFileProvider>();
    let paths = host.services.get_required::<PathManager>();
    let torrent_path = paths.get_source_torrent_path(AlbumConfig::TORRENT_ID);

    // Act
    let result = provider.get(AlbumConfig::TORRENT_ID).await;

    // Assert
    assert!(result.is_err(), "download should have failed");
    assert!(
        !torrent_path.is_file(),
        "no file should remain at final torrent path after failed download, but found: {}",
        torrent_path.display()
    );
    Ok(())
}
