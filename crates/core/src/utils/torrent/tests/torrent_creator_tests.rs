use crate::testing_prelude::*;

#[tokio::test]
async fn create_produces_valid_torrent() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("create_produces_valid_torrent");
    let output_path = test_dir.join("test.torrent");

    // Act
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");

    // Assert
    let torrent = TorrentReader::execute(&output_path)
        .await
        .expect("should read created torrent");
    assert_eq!(torrent.name, album.dir_name());
    assert!(torrent.is_private());
    assert_eq!(torrent.source(), Some("TST"));
    assert_eq!(
        torrent.announce.as_deref(),
        Some("https://example.com/announce")
    );
    assert!(torrent.comment().is_none());
    let created_by = torrent.created_by().expect("should have created by");
    assert!(
        created_by.starts_with("caesura "),
        "unexpected created by: {created_by}"
    );
}

#[tokio::test]
async fn create_includes_all_files() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("create_includes_all_files");
    let output_path = test_dir.join("test.torrent");

    // Act
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");

    // Assert
    let torrent = TorrentReader::execute(&output_path)
        .await
        .expect("should read created torrent");
    let files = torrent
        .files
        .as_ref()
        .expect("should be multi-file torrent");
    let expected_count = album.tracks.len() + 1; // tracks + cover image
    assert_eq!(
        files.len(),
        expected_count,
        "torrent should list all non-hidden files from source directory"
    );
}

#[tokio::test]
async fn create_uppercases_source() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("create_uppercases_source");
    let output_path = test_dir.join("test.torrent");

    // Act
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::Red,
    )
    .await
    .expect("should create torrent");

    // Assert
    let torrent = TorrentReader::execute(&output_path)
        .await
        .expect("should read created torrent");
    assert_eq!(torrent.source(), Some("RED"));
}

#[tokio::test]
async fn duplicate_copies_when_source_matches() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());
    let test_dir = TempDirectory::create("duplicate_copies_when_source_matches");
    let dest_path = test_dir.join("copy.torrent");
    let original = TorrentReader::execute(&source_path)
        .await
        .expect("should read source torrent");
    let announce = original.announce.clone().expect("should have announce");

    // Act
    TorrentCreator::duplicate(&source_path, &dest_path, announce, Indexer::Red)
        .await
        .expect("should duplicate torrent");

    // Assert - copied file should be byte-identical
    let original_bytes = read(&source_path).expect("should read original");
    let copy_bytes = read(&dest_path).expect("should read copy");
    assert_eq!(original_bytes, copy_bytes, "copy should be byte-identical");
}

#[tokio::test]
async fn duplicate_rewrites_when_source_differs() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());
    let test_dir = TempDirectory::create("duplicate_rewrites_when_source_differs");
    let dest_path = test_dir.join("rewritten.torrent");

    // Act
    TorrentCreator::duplicate(
        &source_path,
        &dest_path,
        "https://other-tracker.example.com/announce".to_owned(),
        Indexer::Ops,
    )
    .await
    .expect("should duplicate torrent");

    // Assert
    let rewritten = TorrentReader::execute(&dest_path)
        .await
        .expect("should read rewritten torrent");
    assert_eq!(rewritten.source(), Some("OPS"));
    assert_eq!(
        rewritten.announce.as_deref(),
        Some("https://other-tracker.example.com/announce")
    );
    assert!(rewritten.announce_list.is_none());
    assert_eq!(rewritten.name, album.dir_name());
    assert!(rewritten.is_private());
    assert!(!rewritten.pieces.is_empty(), "pieces should be preserved");
}

#[tokio::test]
async fn duplicate_preserves_pieces() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());
    let test_dir = TempDirectory::create("duplicate_preserves_pieces");
    let dest_path = test_dir.join("rewritten.torrent");
    let original = TorrentReader::execute(&source_path)
        .await
        .expect("should read original");

    // Act
    TorrentCreator::duplicate(
        &source_path,
        &dest_path,
        "https://other.example.com/announce".to_owned(),
        Indexer::Ops,
    )
    .await
    .expect("should duplicate torrent");

    // Assert
    let rewritten = TorrentReader::execute(&dest_path)
        .await
        .expect("should read rewritten torrent");
    assert_eq!(original.pieces, rewritten.pieces);
    assert_eq!(original.piece_length, rewritten.piece_length);
    assert_eq!(original.files, rewritten.files);
}
