use crate::testing_prelude::*;

/// Test that `QueueRemoveCommand` removes an existing item.
#[tokio::test]
async fn queue_rm_command_removes_item() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(SAMPLE_SOURCES_DIR.clone()),
        })
        .build();

    // First add items
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Get one hash to remove
    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let initial_count = items.len();
    assert!(initial_count > 0);
    let hash = *items.keys().next().expect("should have at least one item");

    // Build new host with remove args
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueRemoveArgs {
            queue_rm_hash: Some(hash.to_string()),
        })
        .build();

    // Act
    let rm_command = host.services.get_required::<QueueRemoveCommand>();
    let result = rm_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    assert_eq!(items.len(), initial_count - 1);

    Ok(())
}

/// Test that `QueueRemoveCommand` handles non-existent hash.
#[tokio::test]
async fn queue_rm_command_nonexistent_hash() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueRemoveArgs {
            queue_rm_hash: Some("0000000000000000000000000000000000000000".to_owned()),
        })
        .build();

    let command = host.services.get_required::<QueueRemoveCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(false)));

    Ok(())
}

/// Test that `QueueRemoveCommand` rejects invalid hash format.
#[tokio::test]
async fn queue_rm_command_invalid_hash_format() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(QueueRemoveArgs {
            queue_rm_hash: Some("invalid-hash".to_owned()),
        })
        .build();

    let command = host.services.get_required::<QueueRemoveCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(false)));

    Ok(())
}
