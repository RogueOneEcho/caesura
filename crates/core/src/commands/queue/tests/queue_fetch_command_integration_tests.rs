use crate::testing_prelude::*;
use qbittorrent_api::Response;
use qbittorrent_api::get_torrents::Torrent;
use qbittorrent_api::mock::MockQBittorrentClient;

/// Test that `QueueFetchCommand` adds torrents via qBittorrent API.
///
/// Items are built directly from API data without reading `.torrent` files.
#[tokio::test]
async fn queue_fetch_command_adds_torrents() -> Result<(), TestError> {
    // Arrange
    let mock_torrent = Torrent {
        amount_left: 0,
        ..Torrent::mock()
    };
    let (_test_dir, command, queue) =
        queue_fetch_test_helper(vec!["music".to_owned()], vec![mock_torrent]).await;

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueFetchCommand` skips torrents that are not fully downloaded.
#[tokio::test]
async fn queue_fetch_command_skips_incomplete() -> Result<(), TestError> {
    // Arrange
    let mock_torrent = Torrent {
        amount_left: 1000,
        ..Torrent::mock()
    };
    let (_test_dir, command, queue) =
        queue_fetch_test_helper(vec!["music".to_owned()], vec![mock_torrent]).await;

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Create a host configured for `QueueFetchCommand` testing.
///
/// Returns `TestDirectory` to keep it alive for the test duration.
/// Callers must bind it to a named variable like `_test_dir`, not a bare `_`.
async fn queue_fetch_test_helper(
    categories: Vec<String>,
    mock_torrents: Vec<Torrent>,
) -> (TestDirectory, Ref<QueueFetchCommand>, Ref<Queue>) {
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let mock_client = MockQBittorrentClient::new().with_get_torrents(Response {
        status_code: Some(200),
        result: Some(mock_torrents),
    });
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_mock_torrent_client(mock_client)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueFetchOptions {
            qbit_queue_categories: categories,
        })
        .with_options(QbitOptions::mock())
        .expect_build();
    let command = host.services.get_required::<QueueFetchCommand>();
    let queue = host.services.get_required::<Queue>();
    (test_dir, command, queue)
}

async fn get_items(queue: Ref<Queue>) -> Vec<String> {
    queue
        .get_all()
        .await
        .expect("should be able to get all items")
        .into_values()
        .map(|item| item.name)
        .collect()
}
