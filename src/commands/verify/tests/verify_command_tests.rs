use crate::commands::*;
use crate::hosting::*;
use crate::utils::SourceIssue::UnnecessaryDirectory;
use crate::utils::*;
use rogue_logging::Error;
use std::path::PathBuf;

#[tokio::test]
async fn verify_command_mocked() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();

    // Act
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should get source");
    let status = verifier.execute(&source).await;

    // Assert
    if !status.verified
        && let Some(issues) = &status.issues
    {
        for issue in issues {
            eprintln!("Issue: {issue}");
        }
    }
    assert!(status.verified);
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
