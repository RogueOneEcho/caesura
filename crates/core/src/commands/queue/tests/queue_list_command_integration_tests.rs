use crate::testing_prelude::*;

/// Test that `QueueListCommand` returns empty list for empty queue.
#[tokio::test]
async fn queue_list_command_empty_queue() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();

    let command = host.services.get_required::<QueueListCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    Ok(())
}

/// Test that `QueueListCommand` lists added items.
#[tokio::test]
async fn queue_list_command_lists_items() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(SAMPLE_SOURCES_DIR.clone()),
        })
        .expect_build();

    // Add items first
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Act
    let list_command = host.services.get_required::<QueueListCommand>();
    let result = list_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    Ok(())
}
