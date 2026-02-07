use crate::testing_prelude::*;

/// Test that `QueueAddCommand` adds torrent files from a directory.
#[tokio::test]
async fn queue_add_command_adds_torrent_from_directory() -> Result<(), TestError> {
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let torrent_dir = album.single_torrent_dir();
    let (_test_dir, command, queue) = queue_add_test_helper(torrent_dir.to_path_buf()).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueAddCommand` skips duplicate torrents.
#[tokio::test]
async fn queue_add_command_skips_duplicate() -> Result<(), TestError> {
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let torrent_dir = album.single_torrent_dir();
    let (_test_dir, command, queue) = queue_add_test_helper(torrent_dir.to_path_buf()).await;

    command.execute_cli().await?;
    let items = get_items(queue.clone()).await;
    assert_yaml_snapshot!(items);

    command.execute_cli().await?;
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueAddCommand` returns false for non-existent path.
#[tokio::test]
async fn queue_add_command_nonexistent_path_fails() -> Result<(), TestError> {
    let (_test_dir, command, queue) =
        queue_add_test_helper(PathBuf::from("/nonexistent/path")).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(false)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueAddCommand` handles empty directory.
#[tokio::test]
async fn queue_add_command_empty_directory() -> Result<(), TestError> {
    let temp = TempDirectory::create("empty_torrents");
    let (_test_dir, command, queue) = queue_add_test_helper(temp.to_path_buf()).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Create a host configured for `QueueAddCommand` testing.
///
/// Returns `TestDirectory` to keep it alive for the test duration.
///
/// Callers must bind it to a named variable like `_test_dir`, not a bare `_`:
///
/// ```ignore
/// let (_test_dir, command, queue) = queue_add_test_helper(...).await;
/// ```
///
/// NOT a bare `_` as this discards immediately, deleting the directory mid-test:
///
/// ```ignore
/// let (_, command, queue) = queue_add_test_helper(...).await;
/// ```
async fn queue_add_test_helper(
    queue_add_path: PathBuf,
) -> (TestDirectory, Ref<QueueAddCommand>, Ref<Queue>) {
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(queue_add_path),
        })
        .expect_build();
    let command = host.services.get_required::<QueueAddCommand>();
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
