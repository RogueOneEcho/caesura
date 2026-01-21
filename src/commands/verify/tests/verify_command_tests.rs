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
fn test_subdirectory_checks() {
    // Good source because all flacs share the 'b' subdirectory.
    let result = VerifyCommand::subdirectory_checks(&vec![
        FlacFile::new(PathBuf::from("a/b/c.flac"), &PathBuf::from("a/b")),
        FlacFile::new(PathBuf::from("a/b/d.flac"), &PathBuf::from("a/b")),
    ]);
    assert_eq!(result.len(), 0);

    // Good multi-cd source
    let result = VerifyCommand::subdirectory_checks(&vec![
        FlacFile::new(
            PathBuf::from("/root/album/CD1/a.flac"),
            &PathBuf::from("/root/album/"),
        ),
        FlacFile::new(
            PathBuf::from("/root/album/CD2/a.flac"),
            &PathBuf::from("/root/album/"),
        ),
    ]);
    assert_eq!(result.len(), 0);

    // Bad source because all flacs share the unnecessary 'c' subdirectory.
    let result = VerifyCommand::subdirectory_checks(&vec![FlacFile::new(
        PathBuf::from("a/b/c/d.flac"),
        &PathBuf::from("a/b"),
    )]);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].to_string(),
        UnnecessaryDirectory {
            path: PathBuf::from("c")
        }
        .to_string()
    );
}
