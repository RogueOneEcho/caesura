use std::path::PathBuf;

use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use rogue_logging::Error;

/// Test that `QueueAddCommand` adds torrent files from a directory.
#[tokio::test]
async fn queue_add_command_adds_torrent_from_directory() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(PathBuf::from(SAMPLE_SOURCES_DIR)),
        })
        .build();

    let command = host.services.get_required::<QueueAddCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    assert!(!items.is_empty());
    assert!(items.values().any(|item| item.name.contains("Test Album")));
    assert!(items.values().all(|item| item.indexer == "red"));

    Ok(())
}

/// Test that `QueueAddCommand` skips duplicate torrents.
#[tokio::test]
async fn queue_add_command_skips_duplicate() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(PathBuf::from(SAMPLE_SOURCES_DIR)),
        })
        .build();

    let command = host.services.get_required::<QueueAddCommand>();

    // Act - add twice
    let first = command.execute_cli().await;
    let queue = host.services.get_required::<Queue>();
    let count_after_first = queue.get_all().await?.len();

    let second = command.execute_cli().await;
    let count_after_second = queue.get_all().await?.len();

    // Assert - both succeed but second adds nothing new
    assert!(first.is_ok());
    assert!(second.is_ok());
    assert_eq!(count_after_first, count_after_second);

    Ok(())
}

/// Test that `QueueAddCommand` returns false for non-existent path.
#[tokio::test]
async fn queue_add_command_nonexistent_path_fails() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(PathBuf::from("/nonexistent/path")),
        })
        .build();

    let command = host.services.get_required::<QueueAddCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(false)));
    Ok(())
}

/// Test that `QueueAddCommand` handles empty directory.
#[tokio::test]
async fn queue_add_command_empty_directory() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(TempDirectory::create("empty_torrents")),
        })
        .build();

    let command = host.services.get_required::<QueueAddCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    assert!(items.is_empty());

    Ok(())
}
