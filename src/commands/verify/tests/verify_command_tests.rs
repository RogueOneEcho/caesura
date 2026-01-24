use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use std::path::PathBuf;

use crate::utils::SourceIssue::UnnecessaryDirectory;
use rogue_logging::Error;

#[tokio::test]
async fn verify_command() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        ..TargetOptions::default()
    });
    let host = HostBuilder::new().with_options(target_options).build();
    let verifier = host.services.get_required_mut::<VerifyCommand>();
    let mut verifier = verifier
        .write()
        .expect("verifier should be available to write");

    // Act
    let _is_verified = verifier.execute_cli().await?;

    // Assert not required
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
        FlacFile::new(
            PathBuf::from("source/dir/CD1/a.flac"),
            &source_dir,
        ),
        FlacFile::new(
            PathBuf::from("source/dir/CD2/b.flac"),
            &source_dir,
        ),
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
