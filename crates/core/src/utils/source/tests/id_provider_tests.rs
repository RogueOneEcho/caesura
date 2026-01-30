use crate::hosting::*;
use crate::options::SourceArg;
use crate::utils::*;

const TEST_TORRENT_ID: u32 = 12345;
const TEST_TORRENT_ID_LARGE: u32 = 123_456_789;
const TEST_GROUP_ID: u32 = 11111;

/// Test that `IdProvider` correctly parses a numeric ID string.
#[tokio::test]
async fn id_provider_parses_numeric_id() {
    // Arrange
    let host = create_host_with_source(&TEST_TORRENT_ID.to_string());
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.expect("should have id"), TEST_TORRENT_ID);
}

/// Test that `IdProvider` correctly parses a large numeric ID.
#[tokio::test]
async fn id_provider_parses_large_numeric_id() {
    // Arrange
    let host = create_host_with_source(&TEST_TORRENT_ID_LARGE.to_string());
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.expect("should have id"), TEST_TORRENT_ID_LARGE);
}

/// Test that `IdProvider` correctly parses a URL with group and torrent ID.
#[tokio::test]
async fn id_provider_parses_group_url() {
    // Arrange
    let url =
        format!("https://redacted.sh/torrents.php?id={TEST_GROUP_ID}&torrentid={TEST_TORRENT_ID}");
    let host = create_host_with_source(&url);
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.expect("should have id"), TEST_TORRENT_ID);
}

/// Test that `IdProvider` correctly parses a URL with only torrent ID.
#[tokio::test]
async fn id_provider_parses_torrent_url() {
    // Arrange
    let url = format!("https://redacted.sh/torrents.php?torrentid={TEST_TORRENT_ID}");
    let host = create_host_with_source(&url);
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.expect("should have id"), TEST_TORRENT_ID);
}

/// Test that `IdProvider` correctly parses a URL with hash fragment.
#[tokio::test]
async fn id_provider_parses_url_with_hash() {
    // Arrange
    let url = format!(
        "https://redacted.sh/torrents.php?id={TEST_GROUP_ID}&torrentid={TEST_TORRENT_ID}#torrent{TEST_TORRENT_ID}"
    );
    let host = create_host_with_source(&url);
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.expect("should have id"), TEST_TORRENT_ID);
}

/// Test that `IdProvider` returns `NoMatch` for empty input.
#[tokio::test]
async fn id_provider_empty_input_returns_no_match() {
    // Arrange
    let host = create_host_with_source("");
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result, Err(IdProviderError::NoMatch)));
}

/// Test that `IdProvider` returns `NoMatch` for random text.
#[tokio::test]
async fn id_provider_random_text_returns_no_match() {
    // Arrange
    let host = create_host_with_source("not-a-valid-input");
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result, Err(IdProviderError::NoMatch)));
}

/// Test that `IdProvider` returns `UrlInvalid` for malformed URL.
#[tokio::test]
async fn id_provider_malformed_url_returns_url_invalid() {
    // Arrange
    let host = create_host_with_source("https://redacted.sh/torrents.php?invalid");
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result, Err(IdProviderError::UrlInvalid)));
}

/// Test that `IdProvider` returns `TorrentFileNotFound` for missing torrent file.
#[tokio::test]
async fn id_provider_missing_torrent_file_returns_not_found() {
    // Arrange
    let host = create_host_with_source("/nonexistent/path/to/file.torrent");
    let provider = host.services.get_required::<IdProvider>();

    // Act
    let result = provider.get_by_options().await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result, Err(IdProviderError::TorrentFileNotFound)));
}

/// Helper function to create a host with a specific source argument.
fn create_host_with_source(source: &str) -> Host {
    HostBuilder::new()
        .with_options(SourceArg {
            source: Some(source.to_owned()),
        })
        .build()
}
