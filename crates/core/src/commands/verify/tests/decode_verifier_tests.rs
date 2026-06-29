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
