use crate::testing_prelude::*;
use gazelle_api::{Group, GroupResponse, MockGazelleClient, Torrent, TorrentResponse};

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
