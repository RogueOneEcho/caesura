use std::fs;

use crate::testing_prelude::*;

#[tokio::test]
async fn verify_source_content_passes() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let torrent_path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());
    let content_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());

    // Act
    let result = TorrentVerifier::execute(&torrent_path, &content_dir)
        .await
        .expect("should verify torrent");

    // Assert
    assert!(
        result.is_none(),
        "valid source content should pass verification"
    );
}

#[tokio::test]
async fn verify_transcode_content_passes() {
    // Arrange
    init_logger();
    let config = TranscodeProvider::get(SampleFormat::FLAC16_441, TargetFormat::_320).await;
    let torrent_path = config.torrent_path();
    let content_dir = config.transcode_dir();

    // Act
    let result = TorrentVerifier::execute(&torrent_path, &content_dir)
        .await
        .expect("should verify torrent");

    // Assert
    assert!(
        result.is_none(),
        "valid transcode content should pass verification"
    );
}

#[tokio::test]
async fn verify_corrupted_content_returns_hash_check() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("verify_corrupted_content");
    let output_path = test_dir.join("test.torrent");
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");
    let corrupt_dir = test_dir.join(album.dir_name());
    copy_dir(&source_dir, &corrupt_dir);
    let first_track = album
        .tracks
        .first()
        .expect("should have at least one track");
    let corrupt_path = corrupt_dir.join(album.track_filename(first_track));
    fs::write(&corrupt_path, b"corrupted data").expect("should write corrupt file");

    // Act
    let result = TorrentVerifier::execute(&output_path, &corrupt_dir)
        .await
        .expect("should verify torrent");

    // Assert
    let issue = result.expect("corrupted content should fail verification");
    assert!(
        matches!(issue, SourceIssue::HashCheck { .. }),
        "expected HashCheck issue, got: {issue}"
    );
}

#[tokio::test]
async fn verify_missing_file_returns_missing_file() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("verify_missing_file");
    let output_path = test_dir.join("test.torrent");
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");
    let empty_dir = test_dir.join(album.dir_name());
    fs::create_dir_all(&empty_dir).expect("should create empty dir");

    // Act
    let result = TorrentVerifier::execute(&output_path, &empty_dir)
        .await
        .expect("should verify torrent");

    // Assert
    let issue = result.expect("missing files should fail verification");
    assert!(
        matches!(issue, SourceIssue::MissingFile { .. }),
        "expected MissingFile issue, got: {issue}"
    );
}

#[tokio::test]
async fn verify_truncated_file_returns_hash_check() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let test_dir = TempDirectory::create("verify_truncated_file");
    let output_path = test_dir.join("test.torrent");
    TorrentCreator::create(
        &source_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");
    let truncated_dir = test_dir.join(album.dir_name());
    copy_dir(&source_dir, &truncated_dir);
    let first_track = album
        .tracks
        .first()
        .expect("should have at least one track");
    let track_path = truncated_dir.join(album.track_filename(first_track));
    let original = fs::read(&track_path).expect("should read track file");
    let truncated = original
        .get(..truncate_midpoint(original.len()))
        .expect("slice should be in bounds");
    fs::write(&track_path, truncated).expect("should write truncated file");

    // Act
    let result = TorrentVerifier::execute(&output_path, &truncated_dir)
        .await
        .expect("should verify torrent");

    // Assert
    let issue = result.expect("truncated content should fail verification");
    assert!(
        matches!(issue, SourceIssue::HashCheck { .. }),
        "expected HashCheck issue, got: {issue}"
    );
}

#[tokio::test]
async fn verify_excess_content_returns_excess_content() {
    // Arrange — file size must be an exact multiple of piece_length so all
    // pieces hash correctly and only the trailing bytes trigger ExcessContent.
    init_logger();
    let piece_len: usize = 16384;
    let test_dir = TempDirectory::create("verify_excess_content");
    let content_dir = test_dir.join("album");
    fs::create_dir_all(&content_dir).expect("should create content dir");
    let file_path = content_dir.join("data.bin");
    fs::write(&file_path, vec![0xAB_u8; piece_len]).expect("should write aligned file");
    let output_path = test_dir.join("test.torrent");
    TorrentCreator::create(
        &content_dir,
        &output_path,
        "https://example.com/announce".to_owned(),
        Indexer::from("TST"),
    )
    .await
    .expect("should create torrent");
    let mut content = fs::read(&file_path).expect("should read file");
    content.extend_from_slice(&[0xFF_u8; 512]);
    fs::write(&file_path, &content).expect("should write padded file");

    // Act
    let result = TorrentVerifier::execute(&output_path, &content_dir)
        .await
        .expect("should verify torrent");

    // Assert
    let issue = result.expect("excess content should fail verification");
    assert!(
        matches!(issue, SourceIssue::ExcessContent),
        "expected ExcessContent issue, got: {issue}"
    );
}

#[tokio::test]
async fn verify_nonexistent_torrent_returns_error() {
    // Arrange
    let torrent_path = PathBuf::from("/nonexistent/path.torrent");
    let content_dir = PathBuf::from("/nonexistent/dir");

    // Act
    let result = TorrentVerifier::execute(&torrent_path, &content_dir).await;

    // Assert
    assert!(result.is_err());
}

#[expect(clippy::integer_division, reason = "intentional truncation for test")]
const fn truncate_midpoint(len: usize) -> usize {
    len / 2
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("should create destination dir");
    for entry in fs::read_dir(from).expect("should read source dir") {
        let entry = entry.expect("should read entry");
        let dest = to.join(entry.file_name());
        if entry.file_type().expect("should get file type").is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), &dest).expect("should copy file");
        }
    }
}
