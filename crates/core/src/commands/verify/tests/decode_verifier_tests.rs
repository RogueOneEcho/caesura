use crate::testing_prelude::*;
use std::fs::OpenOptions;

/// A valid FLAC decodes without any issue.
#[tokio::test]
async fn decode_flac_valid() {
    // Arrange
    let path = sample_track().await;
    let source_dir = path
        .parent()
        .expect("track should have a parent directory")
        .to_path_buf();
    let flac = FlacFile::new(path, &source_dir);

    // Act
    let output = decode_flac(&flac.path);

    // Assert
    assert_eq!(output, None);
}

/// A truncated FLAC passes the STREAMINFO header check but fails the full decode.
#[tokio::test]
async fn decode_flac_truncated() {
    // Arrange
    let source = sample_track().await;
    let source_dir = TempDirectory::create("decode_verifier_execute_truncated");
    let path = source_dir.join(source.file_name().expect("track should have a file name"));
    copy(&source, &path).expect("should copy track to isolated directory");
    truncate_to_half(&path);
    let flac = FlacFile::new(path, &source_dir.to_path_buf());
    assert!(
        flac.get_stream_info().is_ok(),
        "header check should still pass on a truncated file"
    );

    // Act
    let output = decode_flac(&flac.path);

    // Assert
    let issue = output.expect("truncated decode should report an issue");
    assert!(
        matches!(issue, SourceIssue::DecodeError { .. }),
        "expected DecodeError, got: {issue}"
    );
}

/// Several valid FLACs plus one truncated FLAC in a single decode pass.
#[tokio::test]
async fn decode_verifier_execute_with_one_truncated() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let decode_verifier = host.services.get_required::<DecodeVerifier>();
    let source_track = sample_track().await;
    let content_dir = TempDirectory::create("decode_verifier_execute_with_one_truncated");
    let valid_one = content_dir.join("01 - valid.flac");
    let valid_two = content_dir.join("02 - valid.flac");
    let truncated = content_dir.join("03 - truncated.flac");
    copy(&source_track, &valid_one).expect("should copy valid one");
    copy(&source_track, &valid_two).expect("should copy valid two");
    copy(&source_track, &truncated).expect("should copy truncated");
    truncate_to_half(&truncated);
    let source_dir = content_dir.to_path_buf();
    let flacs = vec![
        FlacFile::new(valid_one, &source_dir),
        FlacFile::new(valid_two, &source_dir),
        FlacFile::new(truncated.clone(), &source_dir),
    ];

    // Act
    let issues = decode_verifier.execute(&flacs).await;

    // Assert
    assert_eq!(issues.len(), 1, "exactly one decode error expected");
    match issues.first().expect("should have one issue") {
        SourceIssue::DecodeError { path, .. } => assert_eq!(path, &truncated),
        other => unreachable!("expected DecodeError, got: {other}"),
    }
}

/// Path to the single track of the default sample album.
async fn sample_track() -> PathBuf {
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let track = album.tracks.first().expect("album should have a track");
    source_dir.join(album.track_filename(track))
}

/// Truncate a file to half its length, dropping trailing bytes.
#[expect(clippy::integer_division, reason = "halve to truncate")]
fn truncate_to_half(path: &Path) {
    let length = metadata(path).expect("should read metadata").len();
    OpenOptions::new()
        .write(true)
        .open(path)
        .expect("should open file")
        .set_len(length / 2)
        .expect("should truncate file");
}
