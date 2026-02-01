use crate::testing_prelude::*;
use flat_db::Hash;

/// Test that `BatchCommand` succeeds with an empty queue.
#[tokio::test]
async fn batch_command_empty_queue_succeeds() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();

    let command = host.services.get_required::<BatchCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    Ok(())
}

/// Test that `BatchCommand` verifies items and updates queue status.
#[tokio::test]
async fn batch_command_verifies_item() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(album.single_torrent_dir()),
        })
        .expect_build();

    // Add item to queue
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Set the ID on the queue item
    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash)?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let item = queue.get(*hash)?.expect("should have item");
    assert!(item.verify.is_some(), "verify status should be set");
    assert!(
        item.verify.as_ref().expect("checked").verified,
        "item should be verified"
    );

    Ok(())
}

/// Test that `BatchCommand` skips items without an ID.
#[tokio::test]
async fn batch_command_skips_item_without_id() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();

    // Add an item without an ID
    let queue = host.services.get_required::<Queue>();
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    queue
        .set(QueueItem {
            name: "Item Without ID".to_owned(),
            path: PathBuf::from("/test/path.torrent"),
            hash,
            indexer: "red".to_owned(),
            id: None,
            ..QueueItem::default()
        })
        .await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(hash)?.expect("should have item");
    assert!(updated.verify.is_some(), "verify status should be set");
    assert!(
        !updated.verify.as_ref().expect("checked").verified,
        "item without ID should not be verified"
    );

    Ok(())
}

/// Test that `BatchCommand` respects the limit option.
#[tokio::test]
async fn batch_command_respects_limit() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            limit: 1,
            ..BatchOptions::default()
        })
        .expect_build();

    // Add multiple items with IDs
    let queue = host.services.get_required::<Queue>();
    for i in 0..3 {
        let hash = Hash::<20>::from_string(&format!("0{i}00000000000000000000000000000000000000"))?;
        queue
            .set(QueueItem {
                name: format!("Item {i}"),
                path: PathBuf::from(format!("/test/path{i}.torrent")),
                hash,
                indexer: "red".to_owned(),
                id: Some(AlbumConfig::TORRENT_ID),
                ..QueueItem::default()
            })
            .await?;
    }

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let items = queue.get_all().await?;
    let processed = items.values().filter(|i| i.verify.is_some()).count();
    assert_eq!(processed, 1, "only 1 item should be processed due to limit");

    Ok(())
}

/// Test that `BatchCommand` filters by indexer.
#[tokio::test]
async fn batch_command_filters_by_indexer() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();

    let queue = host.services.get_required::<Queue>();

    let red_hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    queue
        .set(QueueItem {
            name: "RED Item".to_owned(),
            path: PathBuf::from("/test/red.torrent"),
            hash: red_hash,
            indexer: "red".to_owned(),
            id: None,
            ..QueueItem::default()
        })
        .await?;

    let ops_hash = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;
    queue
        .set(QueueItem {
            name: "OPS Item".to_owned(),
            path: PathBuf::from("/test/ops.torrent"),
            hash: ops_hash,
            indexer: "ops".to_owned(),
            id: None,
            ..QueueItem::default()
        })
        .await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let red_item = queue.get(red_hash)?.expect("should have RED item");
    let ops_item = queue.get(ops_hash)?.expect("should have OPS item");

    assert!(red_item.verify.is_some(), "RED item should be processed");
    assert!(
        ops_item.verify.is_none(),
        "OPS item should not be processed"
    );

    Ok(())
}

/// Test that `BatchCommand` skips already verified items when transcode is disabled.
#[tokio::test]
async fn batch_command_skips_verified_when_transcode_disabled() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();

    let queue = host.services.get_required::<Queue>();
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    queue
        .set(QueueItem {
            name: "Already Verified".to_owned(),
            path: PathBuf::from("/test/verified.torrent"),
            hash,
            indexer: "red".to_owned(),
            id: Some(123),
            verify: Some(VerifyStatus::verified()),
            ..QueueItem::default()
        })
        .await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let item = queue.get(hash)?.expect("should have item");
    assert!(item.verify.as_ref().expect("checked").verified);

    Ok(())
}

/// Test that `BatchCommand` processes verified items when transcode is enabled.
#[tokio::test]
async fn batch_command_processes_verified_when_transcode_enabled() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            ..BatchOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(album.single_torrent_dir()),
        })
        .expect_build();

    // Add item to queue
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Mark as verified and set ID
    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash)?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash)?.expect("should have item");
    assert!(
        updated.transcode.is_some(),
        "transcode status should be set"
    );

    Ok(())
}

/// Test that `BatchCommand` with `dry_run` does NOT save upload status.
#[tokio::test]
async fn batch_command_upload_dry_run_does_not_save_status() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            upload: true,
            ..BatchOptions::default()
        })
        .with_options(UploadOptions {
            dry_run: true,
            ..UploadOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(album.single_torrent_dir()),
        })
        .expect_build();

    // Add item and set up for processing
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash)?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash)?.expect("should have item");
    assert!(updated.transcode.is_some());
    assert!(
        updated.upload.is_none(),
        "upload status should NOT be saved with dry_run"
    );

    Ok(())
}

/// Test that `BatchCommand` with upload saves status.
#[tokio::test]
async fn batch_command_upload_saves_status() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            upload: true,
            ..BatchOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(album.single_torrent_dir()),
        })
        .expect_build();

    // Add item and set up for processing
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash)?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash)?.expect("should have item");
    assert!(updated.transcode.is_some());
    assert!(updated.upload.is_some(), "upload status should be saved");
    assert!(updated.upload.as_ref().expect("checked").success);

    Ok(())
}
