use crate::testing_prelude::*;

#[tokio::test]
async fn get_finds_directory_with_exact_path() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let _album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let dir_name = AlbumConfig::default().dir_name();
    let host = build_host(&test_dir, mock_api(&dir_name)).await;
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let result = provider.get(AlbumConfig::TORRENT_ID).await;

    // Assert
    let source = result?.expect("should find source with exact path");
    assert_eq!(source.directory, SAMPLE_SOURCES_DIR.join(&dir_name));
    Ok(())
}

#[tokio::test]
async fn get_finds_directory_when_api_path_has_bidi_characters() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let _album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let dir_name = AlbumConfig::default().dir_name();
    let bidi_name = format!("\u{200e}{dir_name}\u{202a}");
    let host = build_host(&test_dir, mock_api(&bidi_name)).await;
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let source = provider.get(AlbumConfig::TORRENT_ID).await;

    // Assert
    let source = source?.expect("should find source via safe path fallback");
    assert_eq!(source.directory, SAMPLE_SOURCES_DIR.join(&dir_name));
    Ok(())
}

#[tokio::test]
async fn get_returns_missing_directory_when_path_not_found() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let _album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let host = build_host(&test_dir, mock_api("Nonexistent Directory")).await;
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let result = provider.get(AlbumConfig::TORRENT_ID).await;

    // Assert
    let inner = result?;
    assert!(matches!(inner, Err(SourceIssue::MissingDirectory { .. })));
    Ok(())
}

const TEST_HASH: &str = "0123456789abcdef0123456789abcdef01234567";

/// Verify `get_from_options` dispatches to hash lookup when the source arg is a hash.
#[tokio::test]
async fn source_provider_get_from_options_dispatches_to_hash() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let _album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let dir_name = AlbumConfig::default().dir_name();
    let host = HostBuilder::new()
        .with_options(SourceArg {
            source: TEST_HASH.to_owned(),
        })
        .with_mock_client(mock_api_with_hash(&dir_name))
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let result = provider.get_from_options().await;

    // Assert
    let source = result?.expect("should find source via hash dispatch");
    assert_eq!(source.directory, SAMPLE_SOURCES_DIR.join(&dir_name));
    Ok(())
}

/// Verify hash dispatch returns `NotFound` when the api has no torrent at the hash.
#[tokio::test]
async fn source_provider_get_from_options_hash_not_found() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let client = MockGazelleClient::new().with_get_torrent_by_hash(Err(GazelleError {
        operation: GazelleOperation::ApiResponse(ApiResponseKind::NotFound),
        source: ErrorSource::ApiResponse(ApiResponseError {
            message: "not found".to_owned(),
            status: 404,
        }),
    }));
    let host = HostBuilder::new()
        .with_options(SourceArg {
            source: TEST_HASH.to_owned(),
        })
        .with_mock_client(client)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();

    // Act
    let result = provider.get_from_options().await;

    // Assert
    assert!(matches!(result?, Err(SourceIssue::NotFound)));
    Ok(())
}

fn mock_api_with_hash(file_path: &str) -> MockGazelleClient {
    let torrent = Torrent {
        id: AlbumConfig::TORRENT_ID,
        file_path: file_path.to_owned(),
        ..Torrent::mock()
    };
    let group = Group::mock();
    MockGazelleClient::new()
        .with_get_torrent_by_hash(Ok(TorrentResponse {
            group: group.clone(),
            torrent: torrent.clone(),
        }))
        .with_get_torrent_group(Ok(GroupResponse {
            group,
            torrents: vec![torrent],
        }))
}

fn mock_api(file_path: &str) -> MockGazelleClient {
    let torrent = Torrent {
        id: AlbumConfig::TORRENT_ID,
        file_path: file_path.to_owned(),
        ..Torrent::mock()
    };
    let group = Group::mock();
    MockGazelleClient::new()
        .with_get_torrent(Ok(TorrentResponse {
            group: group.clone(),
            torrent: torrent.clone(),
        }))
        .with_get_torrent_group(Ok(GroupResponse {
            group,
            torrents: vec![torrent],
        }))
}

async fn build_host(test_dir: &TestDirectory, client: MockGazelleClient) -> Host {
    HostBuilder::new()
        .with_mock_client(client)
        .with_test_options(test_dir)
        .await
        .expect_build()
}
