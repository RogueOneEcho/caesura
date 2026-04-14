use crate::testing_prelude::*;
use crate::utils::SourceIssue::UnnecessaryDirectory;
use std::fs::{self, create_dir};
use std::io::{Error as IoError, ErrorKind};

#[tokio::test]
async fn verify_command_mocked() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();

    // Act
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    let result = verifier.execute(&source).await.expect("should not fail");

    // Assert
    if !result.verified() {
        for issue in &result.issues {
            eprintln!("Issue: {issue}");
        }
    }
    assert!(result.verified());
    Ok(())
}

#[tokio::test]
async fn get_source_torrent_downloads_then_caches() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    let paths = host.services.get_required::<PathManager>();
    let torrent_path = paths.get_source_torrent_path(source.torrent.id);
    assert!(!torrent_path.is_file(), "torrent should not be cached yet");

    // Act
    let result = verifier.get_source_torrent(&source).await?;

    // Assert
    assert_eq!(result, torrent_path);
    assert!(
        torrent_path.is_file(),
        "torrent should be cached after download"
    );
    let metadata = fs::metadata(&torrent_path)?;
    let first_modified = metadata.modified()?;

    // Act
    let result = verifier.get_source_torrent(&source).await?;

    // Assert
    assert_eq!(result, torrent_path);
    let metadata = fs::metadata(&torrent_path)?;
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
async fn get_source_torrent_leaves_no_file_on_download_failure() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let failing_api = album
        .api()
        .with_download_torrent(Err(gazelle_api::GazelleError {
            operation: gazelle_api::GazelleOperation::SendRequest,
            source: gazelle_api::ErrorSource::Io(IoError::new(
                ErrorKind::ConnectionRefused,
                "simulated download failure",
            )),
        }));
    let host = HostBuilder::new()
        .with_mock_client(failing_api)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    let paths = host.services.get_required::<PathManager>();
    let torrent_path = paths.get_source_torrent_path(source.torrent.id);

    // Act
    let result = verifier.get_source_torrent(&source).await;

    // Assert
    assert!(result.is_err(), "download should have failed");
    assert!(
        !torrent_path.is_file(),
        "no file should remain at final torrent path after failed download, but found: {}",
        torrent_path.display()
    );
    Ok(())
}

#[tokio::test]
async fn hash_check_returns_issue_on_mismatch() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();
    let mut source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    // Download the torrent first so the cache is populated
    verifier
        .get_source_torrent(&source)
        .await
        .expect("should download");
    // Point directory at an empty dir so hashes won't match
    let empty_dir = test_dir.join("empty");
    create_dir(&empty_dir)?;
    source.directory = empty_dir;

    // Act
    let result = verifier.hash_check(&source).await;

    // Assert
    let issue = result
        .expect("hash check should not return infrastructure failure")
        .expect("hash check should return an issue");
    assert!(
        matches!(issue, SourceIssue::MissingFile { .. }),
        "expected MissingFile, got: {issue}"
    );
    Ok(())
}

#[test]
#[allow(clippy::indexing_slicing)]
fn test_subdirectory_checks() {
    let source_dir = PathBuf::from("source/dir");

    // Good source because all flacs share the 'b' subdirectory.
    let result = VerifyCommand::subdirectory_checks(&[
        FlacFile::new(PathBuf::from("source/dir/a.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/b.flac"), &source_dir),
    ]);
    assert_eq!(result.len(), 0);

    // Bad source because all flacs share the 'c' subdirectory.
    let result = VerifyCommand::subdirectory_checks(&[
        FlacFile::new(PathBuf::from("source/dir/a/b.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/a/c.flac"), &source_dir),
    ]);
    assert_eq!(result.len(), 1);

    // Good multi-cd source
    let result = VerifyCommand::subdirectory_checks(&[
        FlacFile::new(PathBuf::from("source/dir/CD1/a.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/CD2/b.flac"), &source_dir),
    ]);
    assert_eq!(result.len(), 0);

    // Bad source because all flacs share the unnecessary 'c' subdirectory.
    let result = VerifyCommand::subdirectory_checks(&[FlacFile::new(
        PathBuf::from("source/dir/c/d.flac"),
        &source_dir,
    )]);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].to_string(),
        UnnecessaryDirectory {
            prefix: PathBuf::from("c")
        }
        .to_string()
    );

    // Good single-file release directly in source directory
    let result = VerifyCommand::subdirectory_checks(&[FlacFile::new(
        PathBuf::from("/root/album/track.flac"),
        &PathBuf::from("/root/album/"),
    )]);
    assert_eq!(result.len(), 0);
}

#[test]
fn check_directory_exists_exists() {
    // Arrange
    let mut source = mock_source();
    source.directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Act + Assert
    assert!(check_directory_exists(&source).is_none());
}

#[test]
fn check_directory_exists_missing() {
    // Arrange
    let mut source = mock_source();
    source.directory = PathBuf::from("/nonexistent/path");

    // Act + Assert
    assert_eq!(
        check_directory_exists(&source),
        Some(SourceIssue::MissingDirectory {
            path: PathBuf::from("/nonexistent/path")
        })
    );
}

#[test]
fn check_flac_count_matches() {
    // Arrange
    let source = mock_source();
    let actual = source.torrent.get_flacs().len();

    // Act + Assert
    assert!(check_flac_count(&source, actual).is_none());
}

#[test]
fn check_flac_count_mismatch() {
    // Arrange
    let source = mock_source();

    // Act + Assert
    assert_eq!(
        check_flac_count(&source, 5),
        Some(SourceIssue::FlacCount {
            expected: 1,
            actual: 5
        })
    );
}

#[test]
fn check_path_length_within_limit() {
    let path = PathBuf::from("a".repeat(180));
    assert_eq!(check_path_length(&path), None);
}

#[test]
fn check_path_length_exceeds_limit() {
    let path = PathBuf::from("a".repeat(185));
    assert_eq!(
        check_path_length(&path),
        Some(SourceIssue::Length {
            path: path.clone(),
            excess: 5,
        })
    );
}

fn mock_source() -> Source {
    Source {
        torrent: gazelle_api::Torrent::mock(),
        group: gazelle_api::Group::mock(),
        targets: TargetFormat::all(),
        format: SourceFormat::Flac,
        directory: PathBuf::from("/tmp/test"),
        metadata: Metadata::new(&gazelle_api::Group::mock(), &gazelle_api::Torrent::mock()),
        url: get_permalink(&RED_URL.to_owned(), 123, 456),
    }
}
