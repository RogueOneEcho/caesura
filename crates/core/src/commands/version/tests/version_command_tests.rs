use crate::commands::version::get_version;
use crate::testing_prelude::*;

#[tokio::test]
async fn execute_returns_true_when_dependencies_present() {
    // Arrange
    init_logger();
    let host = HostBuilder::new().expect_build();
    let command = host.services.get_required::<VersionCommand>();

    // Act
    let result = command.execute().await;

    // Assert
    assert!(result, "should return true when all dependencies are found");
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
