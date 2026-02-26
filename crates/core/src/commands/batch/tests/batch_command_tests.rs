use crate::testing_prelude::*;
use flat_db::Hash;
use std::fs;

/// Test that `BatchCommand` succeeds with an empty queue.
#[tokio::test]
async fn batch_command_empty_queue_succeeds() -> Result<(), TestError> {
    // Arrange
    init_logger();
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
async fn batch_command_verifies_item() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(QueueAddArgs {
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    // Add item to queue
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Set the ID on the queue item
    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let item = queue.get(*hash).await?.expect("should have item");
    assert!(item.verify.is_some(), "verify status should be set");
    assert!(
        item.verify.as_ref().expect("checked").verified,
        "item should be verified"
    );

    Ok(())
}

/// Test that `BatchCommand` skips items without an ID.
#[tokio::test]
async fn batch_command_skips_item_without_id() -> Result<(), TestError> {
    // Arrange
    init_logger();
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

    let updated = queue.get(hash).await?.expect("should have item");
    assert!(updated.verify.is_some(), "verify status should be set");
    assert!(
        !updated.verify.as_ref().expect("checked").verified,
        "item without ID should not be verified"
    );

    Ok(())
}

/// Test that `BatchCommand` respects the limit option.
#[tokio::test]
async fn batch_command_respects_limit() -> Result<(), TestError> {
    // Arrange
    init_logger();
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
async fn batch_command_filters_by_indexer() -> Result<(), TestError> {
    // Arrange
    init_logger();
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

    let red_item = queue.get(red_hash).await?.expect("should have RED item");
    let ops_item = queue.get(ops_hash).await?.expect("should have OPS item");

    assert!(red_item.verify.is_some(), "RED item should be processed");
    assert!(
        ops_item.verify.is_none(),
        "OPS item should not be processed"
    );

    Ok(())
}

/// Test that `BatchCommand` skips already verified items when transcode is disabled.
#[tokio::test]
async fn batch_command_skips_verified_when_transcode_disabled() -> Result<(), TestError> {
    // Arrange
    init_logger();
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

    let item = queue.get(hash).await?.expect("should have item");
    assert!(item.verify.as_ref().expect("checked").verified);

    Ok(())
}

/// Test that `BatchCommand` processes verified items when transcode is enabled.
#[tokio::test]
async fn batch_command_processes_verified_when_transcode_enabled() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            ..BatchOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    // Add item to queue
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    // Mark as verified and set ID
    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash).await?.expect("should have item");
    assert!(
        updated.transcode.is_some(),
        "transcode status should be set"
    );

    Ok(())
}

/// Test that `BatchCommand` with `dry_run` does NOT save upload status.
#[tokio::test]
async fn batch_command_upload_dry_run_does_not_save_status() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();

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
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    // Add item and set up for processing
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash).await?.expect("should have item");
    assert!(updated.transcode.is_some());
    assert!(
        updated.upload.is_none(),
        "upload status should NOT be saved with dry_run"
    );

    Ok(())
}

/// Test that `BatchCommand` with upload saves status.
#[tokio::test]
async fn batch_command_upload_saves_status() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();

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
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    // Add item and set up for processing
    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));

    let updated = queue.get(*hash).await?.expect("should have item");
    assert!(updated.transcode.is_some());
    assert!(updated.upload.is_some(), "upload status should be saved");
    assert!(updated.upload.as_ref().expect("checked").success);

    Ok(())
}

/// Test that `BatchCommand` executes `post_transcode_hook` with a payload.
#[tokio::test]
async fn batch_command_post_transcode_hook_receives_payload() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();
    let hook_dir = TempDirectory::create("batch_post_transcode_hook");
    let hook_path = hook_dir.join("post-transcode-hook.sh");
    let hook_payload_path = hook_dir.join("post-transcode-payload.yml");
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\ncp \"$1\" \"{}\"\n",
        hook_payload_path.display()
    );
    fs::write(&hook_path, script)?;

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            post_transcode_hook: Some(hook_path),
            ..BatchOptions::default()
        })
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    assert!(
        hook_payload_path.is_file(),
        "post-transcode hook should write payload file"
    );
    let payload_yaml = fs::read_to_string(&hook_payload_path)?;
    let payload: serde_yaml::Value = serde_yaml::from_str(&payload_yaml)?;
    assert!(
        payload
            .get("torrent_id")
            .is_some_and(serde_yaml::Value::is_null)
    );
    assert!(
        payload
            .get("group_id")
            .and_then(serde_yaml::Value::as_u64)
            .is_some()
    );
    assert!(
        payload
            .get("permalink")
            .is_some_and(serde_yaml::Value::is_null)
    );
    assert!(
        payload
            .get("source_name")
            .and_then(serde_yaml::Value::as_str)
            .is_some()
    );
    assert!(
        payload
            .get("source_path")
            .and_then(serde_yaml::Value::as_str)
            .is_some()
    );
    let transcode_path = payload
        .get("transcode_path")
        .and_then(serde_yaml::Value::as_str)
        .expect("transcode_path should be set");
    assert!(
        Path::new(transcode_path).is_dir(),
        "transcode_path should exist: {transcode_path}"
    );
    let torrent_path = payload
        .get("torrent_path")
        .and_then(serde_yaml::Value::as_str)
        .expect("torrent_path should be set");
    assert!(
        Path::new(torrent_path).is_file(),
        "torrent_path should exist: {torrent_path}"
    );

    Ok(())
}

/// Test that `BatchCommand` executes `post_upload_hook` with upload metadata.
#[tokio::test]
async fn batch_command_post_upload_hook_receives_payload() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();
    let hook_dir = TempDirectory::create("batch_post_upload_hook");
    let hook_path = hook_dir.join("post-upload-hook.sh");
    let hook_payload_path = hook_dir.join("post-upload-payload.yml");
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\ncp \"$1\" \"{}\"\n",
        hook_payload_path.display()
    );
    fs::write(&hook_path, script)?;

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            upload: true,
            post_upload_hook: Some(hook_path),
            ..BatchOptions::default()
        })
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    assert!(
        hook_payload_path.is_file(),
        "post-upload hook should write payload file"
    );
    let payload_yaml = fs::read_to_string(&hook_payload_path)?;
    let payload: serde_yaml::Value = serde_yaml::from_str(&payload_yaml)?;
    assert!(
        payload
            .get("torrent_id")
            .and_then(serde_yaml::Value::as_u64)
            .is_some()
    );
    assert!(
        payload
            .get("group_id")
            .and_then(serde_yaml::Value::as_u64)
            .is_some()
    );
    let permalink = payload
        .get("permalink")
        .and_then(serde_yaml::Value::as_str)
        .expect("permalink should be set after upload");
    assert!(permalink.contains("torrentid="));
    let transcode_path = payload
        .get("transcode_path")
        .and_then(serde_yaml::Value::as_str)
        .expect("transcode_path should be set");
    assert!(
        Path::new(transcode_path).is_dir(),
        "transcode_path should exist: {transcode_path}"
    );
    let torrent_path = payload
        .get("torrent_path")
        .and_then(serde_yaml::Value::as_str)
        .expect("torrent_path should be set");
    assert!(
        Path::new(torrent_path).is_file(),
        "torrent_path should exist: {torrent_path}"
    );

    Ok(())
}

/// Test that dry-run batch upload skips hook side effects.
#[tokio::test]
async fn batch_command_dry_run_skips_hook_side_effects() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let torrent_dir = album.single_torrent_dir();
    let hook_dir = TempDirectory::create("batch_dry_run_hooks");

    let post_transcode_hook_path = hook_dir.join("post-transcode-hook.sh");
    let post_transcode_payload = hook_dir.join("post-transcode-payload.yml");
    let post_transcode_script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\ncp \"$1\" \"{}\"\n",
        post_transcode_payload.display()
    );
    fs::write(&post_transcode_hook_path, post_transcode_script)?;

    let post_upload_hook_path = hook_dir.join("post-upload-hook.sh");
    let post_upload_payload = hook_dir.join("post-upload-payload.yml");
    let post_upload_script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\ncp \"$1\" \"{}\"\n",
        post_upload_payload.display()
    );
    fs::write(&post_upload_hook_path, post_upload_script)?;

    let host = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(BatchOptions {
            transcode: true,
            upload: true,
            post_transcode_hook: Some(post_transcode_hook_path),
            post_upload_hook: Some(post_upload_hook_path),
            ..BatchOptions::default()
        })
        .with_options(UploadOptions {
            dry_run: true,
            ..UploadOptions::default()
        })
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .with_options(QueueAddArgs {
            queue_add_path: Some(torrent_dir.to_path_buf()),
        })
        .expect_build();

    let add_command = host.services.get_required::<QueueAddCommand>();
    add_command.execute_cli().await?;

    let queue = host.services.get_required::<Queue>();
    let items = queue.get_all().await?;
    let (hash, _) = items.iter().next().expect("should have item");
    let mut item = queue.get(*hash).await?.expect("should have item");
    item.id = Some(AlbumConfig::TORRENT_ID);
    item.verify = Some(VerifyStatus::verified());
    queue.set(item).await?;

    let batch_command = host.services.get_required::<BatchCommand>();

    // Act
    let result = batch_command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(true)));
    assert!(
        !post_transcode_payload.exists(),
        "post-transcode hook should be skipped during dry run"
    );
    assert!(
        !post_upload_payload.exists(),
        "post-upload hook should be skipped during dry run"
    );

    Ok(())
}
