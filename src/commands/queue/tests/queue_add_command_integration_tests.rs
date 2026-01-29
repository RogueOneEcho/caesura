use std::path::PathBuf;

use di::Ref;
use insta::assert_yaml_snapshot;

use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use rogue_logging::Error;

/// Test that `QueueAddCommand` adds torrent files from a directory.
#[tokio::test]
async fn queue_add_command_adds_torrent_from_directory() -> Result<(), Error> {
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let (command, queue) = queue_add_test_helper(album.single_torrent_dir()).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueAddCommand` skips duplicate torrents.
#[tokio::test]
async fn queue_add_command_skips_duplicate() -> Result<(), Error> {
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let (command, queue) = queue_add_test_helper(album.single_torrent_dir()).await;

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
async fn queue_add_command_nonexistent_path_fails() -> Result<(), Error> {
    let (command, queue) = queue_add_test_helper(PathBuf::from("/nonexistent/path")).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(false)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}

/// Test that `QueueAddCommand` handles empty directory.
#[tokio::test]
async fn queue_add_command_empty_directory() -> Result<(), Error> {
    let (command, queue) = queue_add_test_helper(TempDirectory::create("empty_torrents")).await;

    let result = command.execute_cli().await;

    assert!(matches!(result, Ok(true)));
    let items = get_items(queue).await;
    assert_yaml_snapshot!(items);
    Ok(())
}
/// Create a host configured for `QueueAddCommand` testing.
async fn queue_add_test_helper(queue_add_path: PathBuf) -> (Ref<QueueAddCommand>, Ref<Queue>) {
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(queue_add_path),
        })
        .build();
    let command = host.services.get_required::<QueueAddCommand>();
    let queue = host.services.get_required::<Queue>();
    (command, queue)
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
