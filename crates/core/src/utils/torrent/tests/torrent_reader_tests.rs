use crate::testing_prelude::*;

#[tokio::test]
async fn read_source_torrent() {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());

    // Act
    let torrent = TorrentReader::execute(&path)
        .await
        .expect("should read torrent");

    // Assert
    assert_eq!(torrent.name, album.dir_name());
    assert!(torrent.is_private());
}

#[tokio::test]
async fn read_source_torrent_has_source() {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());

    // Act
    let torrent = TorrentReader::execute(&path)
        .await
        .expect("should read torrent");

    // Assert
    assert_eq!(torrent.source(), Some("RED"));
}

#[tokio::test]
async fn read_source_torrent_has_created_by() {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());

    // Act
    let torrent = TorrentReader::execute(&path)
        .await
        .expect("should read torrent");

    // Assert
    let created_by = torrent.created_by().expect("should have created by");
    assert!(
        created_by.starts_with("caesura "),
        "unexpected created by: {created_by}"
    );
}

#[tokio::test]
async fn read_source_torrent_has_files() {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());

    // Act
    let torrent = TorrentReader::execute(&path)
        .await
        .expect("should read torrent");

    // Assert
    let files = torrent
        .files
        .as_ref()
        .expect("should be multi-file torrent");
    assert!(!files.is_empty(), "torrent should contain files");
    let flac_count = files
        .iter()
        .filter(|f| f.path.extension().is_some_and(|ext| ext == "flac"))
        .count();
    assert_eq!(flac_count, album.tracks.len());
}

#[tokio::test]
async fn read_transcode_torrent() {
    // Arrange
    init_logger();
    let config = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::_320).await;
    let path = config.torrent_path();

    // Act
    let torrent = TorrentReader::execute(&path)
        .await
        .expect("should read torrent");

    // Assert
    assert_eq!(torrent.name, config.dir_name());
    assert!(torrent.is_private());
    assert_eq!(torrent.source(), Some("RED"));
    let files = torrent
        .files
        .as_ref()
        .expect("should be multi-file torrent");
    let mp3_count = files
        .iter()
        .filter(|f| f.path.extension().is_some_and(|ext| ext == "mp3"))
        .count();
    assert!(mp3_count > 0, "torrent should contain mp3 files");
}

#[tokio::test]
async fn read_nonexistent_torrent_returns_error() {
    // Arrange
    let path = PathBuf::from("/nonexistent/path.torrent");

    // Act
    let result = TorrentReader::execute(&path).await;

    // Assert
    assert!(result.is_err());
}
