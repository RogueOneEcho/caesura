use crate::commands::version::{DependencyVersions, VersionError, VersionInfo, get_version};
use crate::testing_prelude::*;

#[tokio::test]
async fn version_command_get_versions() {
    // Arrange
    let host = HostBuilder::mock().expect_build();
    let command = host.services.get_required::<VersionCommand>();

    // Act
    let versions = command.get_versions().await;

    // Assert
    assert!(versions.check_versions(), "should find all dependencies");
}

#[test]
fn dependency_versions_check_versions_missing() {
    // Arrange
    let versions = DependencyVersions([
        (FLAC, Ok(VersionInfo::mock())),
        (LAME, Err(VersionError::EmptyStdout)),
        (SOX, Ok(VersionInfo::mock())),
    ]);

    // Act
    let found = versions.check_versions();

    // Assert
    assert!(!found);
}

#[tokio::test]
async fn get_version_flac() {
    // Act
    let info = get_version(FLAC, FLAC_VERSION_PATTERN)
        .await
        .expect("flac should be available");

    // Assert
    assert!(info.version.is_some(), "should extract flac version");
    assert!(!info.first_line.is_empty());
}

#[tokio::test]
async fn get_version_lame() {
    // Act
    let info = get_version(LAME, LAME_VERSION_PATTERN)
        .await
        .expect("lame should be available");

    // Assert
    assert!(info.version.is_some(), "should extract lame version");
    assert!(!info.first_line.is_empty());
}

#[tokio::test]
async fn get_version_sox() {
    // Arrange
    let binary = if detect_sox_ng() { SOX_NG } else { SOX };

    // Act
    let info = get_version(binary, SOX_VERSION_PATTERN)
        .await
        .expect("sox should be available");

    // Assert
    assert!(info.version.is_some(), "should extract sox version");
    assert!(!info.first_line.is_empty());
}

#[tokio::test]
async fn get_version_missing_binary() {
    // Act
    let result = get_version("nonexistent_binary_xyz", r"v(\d+)").await;

    // Assert
    assert!(result.is_err());
}
