use crate::testing_prelude::*;
use gazelle_api::{Group, GroupResponse, MockGazelleClient, Torrent, UploadResponse};
use std::fs;

#[test]
fn publish_manifest_valid_new_group_parses() -> Result<(), TestError> {
    // Arrange
    let source_dir = TempDirectory::create("publish_manifest_valid_new_group_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "test")?;
    let manifest = create_new_group_manifest(source_path);
    let manifest_dir = TempDirectory::create("publish_manifest_valid_new_group");
    let file = manifest_dir.join("publish.new-group.yml");
    fs::write(&file, serde_yaml::to_string(&manifest)?)?;

    // Act
    let parsed = PublishManifest::read(&file)?;

    // Assert
    assert_eq!(parsed.mode, PublishMode::NewGroup);
    assert!(parsed.new_group.is_some());
    assert!(parsed.existing_group.is_none());
    parsed.validate()?;
    Ok(())
}

#[test]
fn publish_manifest_valid_existing_group_parses() -> Result<(), TestError> {
    // Arrange
    let source_dir = TempDirectory::create("publish_manifest_valid_existing_group_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "test")?;
    let manifest = create_existing_group_manifest(source_path);
    let manifest_dir = TempDirectory::create("publish_manifest_valid_existing_group");
    let file = manifest_dir.join("publish.existing-group.yml");
    fs::write(&file, serde_yaml::to_string(&manifest)?)?;

    // Act
    let parsed = PublishManifest::read(&file)?;

    // Assert
    assert_eq!(parsed.mode, PublishMode::ExistingGroup);
    assert!(parsed.new_group.is_none());
    assert!(parsed.existing_group.is_some());
    parsed.validate()?;
    Ok(())
}

#[test]
fn publish_manifest_missing_manual_checks_ack_fails() {
    // Arrange
    let source_dir = TempDirectory::create("publish_manifest_missing_manual_checks_ack_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "test").expect("should write flac file");
    let manifest = PublishManifest {
        manual_checks_ack: false,
        ..create_new_group_manifest(source_path)
    };

    // Act
    let result = manifest.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn publish_manifest_mode_section_mismatch_fails() {
    // Arrange
    let source_dir = TempDirectory::create("publish_manifest_mode_section_mismatch_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "test").expect("should write flac file");
    let mut manifest = create_new_group_manifest(source_path);
    manifest.mode = PublishMode::ExistingGroup;

    // Act
    let result = manifest.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn publish_manifest_invalid_source_path_fails() {
    // Arrange
    let source_path = PathBuf::from("/path/that/does/not/exist");
    let manifest = create_new_group_manifest(source_path);

    // Act
    let result = manifest.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn publish_manifest_source_without_flac_fails() {
    // Arrange
    let dir = TempDirectory::create("publish_manifest_empty_source");
    let manifest = create_new_group_manifest(dir.to_path_buf());

    // Act
    let result = manifest.validate();

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn publish_rejects_non_red_indexer() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let manifest = create_new_group_manifest(source_path);
    let publish_dir = TempDirectory::create("publish_rejects_non_red_indexer");
    let publish_path = publish_dir.join("publish.yml");
    fs::write(&publish_path, serde_yaml::to_string(&manifest)?)?;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg { publish_path })
        .with_options(SharedOptions {
            announce_url: "https://home.opsfet.ch/test/announce".to_owned(),
            indexer: "ops".to_owned(),
            indexer_url: "https://orpheus.network".to_owned(),
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: test_dir.output(),
            ..SharedOptions::mock()
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn publish_dry_run_new_group_skips_api_call() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let manifest = PublishManifest {
        dry_run: true,
        ..create_new_group_manifest(source_path)
    };
    let publish_dir = TempDirectory::create("publish_dry_run_new_group_skips_api_call");
    let publish_path = publish_dir.join("publish.yml");
    fs::write(&publish_path, serde_yaml::to_string(&manifest)?)?;
    let test_dir = TestDirectory::new();
    let mock = MockGazelleClient::new();
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg { publish_path })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn publish_new_group_returns_response_and_next_steps() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let manifest = create_new_group_manifest(source_path);
    manifest.validate()?;
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: Some(364_781),
        torrent_id: 500_001,
        group_id: 600_001,
    };
    let expected_group_id = response.group_id;
    let expected_torrent_id = response.torrent_id;
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    let actual = result.response.expect("response should be present");
    assert_eq!(actual.group_id, expected_group_id);
    assert_eq!(actual.torrent_id, expected_torrent_id);
    assert_eq!(
        result.permalink,
        Some(get_permalink(
            "https://redacted.sh",
            expected_group_id,
            expected_torrent_id
        ))
    );
    assert_eq!(
        result.next_transcode,
        Some(format!("caesura transcode {expected_torrent_id}"))
    );
    assert_eq!(
        result.next_upload,
        Some(format!("caesura upload {expected_torrent_id}"))
    );
    Ok(())
}

#[tokio::test]
async fn publish_existing_group_duplicate_detected_fails() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let manifest = create_existing_group_manifest(source_path);
    manifest.validate()?;
    let test_dir = TestDirectory::new();
    let mock = MockGazelleClient::new().with_get_torrent_group(Ok(GroupResponse {
        group: Group {
            id: 123_456,
            ..Group::default()
        },
        torrents: vec![Torrent {
            media: "WEB".to_owned(),
            format: "FLAC".to_owned(),
            encoding: "Lossless".to_owned(),
            remaster_year: Some(2024),
            remaster_title: "Digital".to_owned(),
            remaster_record_label: "Label".to_owned(),
            remaster_catalogue_number: "CAT-001".to_owned(),
            ..Torrent::default()
        }],
    }));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await;

    // Assert
    assert!(result.is_err());
    let error = result.expect_err("should fail");
    assert!(error.render().contains("duplicate"));
    Ok(())
}

#[tokio::test]
async fn publish_existing_group_non_duplicate_uploads() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());
    let manifest = create_existing_group_manifest(source_path);
    manifest.validate()?;
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 700_001,
        group_id: 123_456,
    };
    let expected_group_id = response.group_id;
    let expected_torrent_id = response.torrent_id;
    let mock = MockGazelleClient::new()
        .with_get_torrent_group(Ok(GroupResponse {
            group: Group {
                id: 123_456,
                ..Group::default()
            },
            torrents: vec![Torrent {
                media: "WEB".to_owned(),
                format: "MP3".to_owned(),
                encoding: "320".to_owned(),
                remaster_year: Some(2024),
                remaster_title: "Digital".to_owned(),
                remaster_record_label: "Label".to_owned(),
                remaster_catalogue_number: "CAT-001".to_owned(),
                ..Torrent::default()
            }],
        }))
        .with_upload_torrent(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    let actual = result.response.expect("response should be present");
    assert_eq!(actual.group_id, expected_group_id);
    assert_eq!(actual.torrent_id, expected_torrent_id);
    assert_eq!(
        result.permalink,
        Some(get_permalink(
            "https://redacted.sh",
            expected_group_id,
            expected_torrent_id
        ))
    );
    assert_eq!(
        result.next_transcode,
        Some(format!("caesura transcode {expected_torrent_id}"))
    );
    assert_eq!(
        result.next_upload,
        Some(format!("caesura upload {expected_torrent_id}"))
    );
    Ok(())
}

#[tokio::test]
async fn publish_stages_source_with_hard_links_by_default() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_stages_source_with_hard_links_by_default");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path.clone());
    manifest.validate()?;
    let content_dir = TempDirectory::create("publish_hard_link_content");
    let output_dir = TempDirectory::create("publish_hard_link_output");
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 500_111,
        group_id: 600_111,
    };
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    assert!(result.response.is_some(), "publish should succeed");
    let staged_path = content_dir.join(
        source_path
            .file_name()
            .expect("source path should have file name"),
    );
    assert!(source_path.is_dir(), "original source should remain");
    assert!(staged_path.is_dir(), "staged source should exist");
    assert_eq!(
        fs::read(source_path.join("01 Track.flac"))?,
        fs::read(staged_path.join("01 Track.flac"))?
    );
    Ok(())
}

#[tokio::test]
async fn publish_move_source_moves_source_directory() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_move_source_moves_source_directory");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path.clone());
    manifest.validate()?;
    let content_dir = TempDirectory::create("publish_move_source_content");
    let output_dir = TempDirectory::create("publish_move_source_output");
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 500_112,
        group_id: 600_112,
    };
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(PublishSeedingOptions { move_source: true })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    assert!(result.response.is_some(), "publish should succeed");
    let staged_path = content_dir.join(
        source_path
            .file_name()
            .expect("source path should have file name"),
    );
    assert!(!source_path.exists(), "source should be moved");
    assert!(staged_path.is_dir(), "staged source should exist");
    assert!(staged_path.join("01 Track.flac").is_file());
    Ok(())
}

#[tokio::test]
async fn publish_source_already_staged_is_not_moved() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let content_dir = TempDirectory::create("publish_source_already_staged_content");
    let source_path = content_dir.join("already-staged-source");
    fs::create_dir_all(&source_path)?;
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path.clone());
    manifest.validate()?;
    let output_dir = TempDirectory::create("publish_source_already_staged_output");
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 500_113,
        group_id: 600_113,
    };
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(PublishSeedingOptions { move_source: true })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    assert!(result.response.is_some(), "publish should succeed");
    assert!(source_path.is_dir(), "source should still exist");
    assert!(source_path.join("01 Track.flac").is_file());
    Ok(())
}

#[tokio::test]
async fn publish_existing_staging_target_fails_before_upload() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_parent = TempDirectory::create("publish_existing_staging_target_source_parent");
    let source_path = source_parent.join("source-for-staging-collision");
    fs::create_dir_all(&source_path)?;
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path.clone());
    manifest.validate()?;
    let content_dir = TempDirectory::create("publish_existing_staging_target_content");
    let output_dir = TempDirectory::create("publish_existing_staging_target_output");
    let collision_path = content_dir.join("source-for-staging-collision");
    fs::create_dir_all(&collision_path)?;
    fs::write(collision_path.join("01 Track.flac"), "existing")?;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_client(MockGazelleClient::new())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await;

    // Assert
    let error = result.expect_err("publish should fail before upload");
    assert_eq!(error.action(), &PublishAction::StageSource);
    Ok(())
}

#[tokio::test]
async fn publish_verify_seed_content_failure_is_reported() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_verify_seed_content_failure_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let mismatched_dir = TempDirectory::create("publish_verify_seed_content_failure_mismatch");
    let mismatched_path = mismatched_dir.to_path_buf();
    fs::write(mismatched_path.join("01 Track.flac"), "different track")?;
    let output_dir = TempDirectory::create("publish_verify_seed_content_failure_output");
    let torrent_path = output_dir.join("source.red.source.torrent");
    TorrentCreator::create(
        &source_path,
        &torrent_path,
        "https://flacsfor.me/test/announce".to_owned(),
        "red".to_owned(),
    )
    .await?;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_client(MockGazelleClient::new())
        .with_test_options(&test_dir)
        .await
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command
        .verify_seed_content(&torrent_path, &mismatched_path)
        .await;

    // Assert
    let error = result.expect_err("verification should fail");
    assert_eq!(error.action(), &PublishAction::VerifySeedContent);
    Ok(())
}

#[tokio::test]
async fn publish_copies_torrent_to_injection_directory() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_copies_torrent_to_injection_directory_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path.clone());
    manifest.validate()?;
    let content_dir =
        TempDirectory::create("publish_copies_torrent_to_injection_directory_content");
    let output_dir = TempDirectory::create("publish_copies_torrent_to_injection_directory_output");
    let injection_dir =
        TempDirectory::create("publish_copies_torrent_to_injection_directory_injection");
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 500_114,
        group_id: 600_114,
    };
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(TorrentInjectionOptions {
            copy_torrent_to: Some(injection_dir.to_path_buf()),
        })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    assert!(result.response.is_some(), "publish should succeed");
    let copied_torrents: Vec<_> = fs::read_dir(&*injection_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "torrent"))
        .collect();
    assert_eq!(copied_torrents.len(), 1, "one torrent should be injected");
    Ok(())
}

#[tokio::test]
async fn publish_injection_failure_is_non_fatal() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_injection_failure_is_non_fatal_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = create_new_group_manifest(source_path);
    manifest.validate()?;
    let content_dir = TempDirectory::create("publish_injection_failure_is_non_fatal_content");
    let output_dir = TempDirectory::create("publish_injection_failure_is_non_fatal_output");
    let injection_dir = TempDirectory::create("publish_injection_failure_is_non_fatal_injection");
    let injection_path = injection_dir.to_path_buf();
    let test_dir = TestDirectory::new();
    let response = UploadResponse {
        private: true,
        source: true,
        request_id: None,
        torrent_id: 500_115,
        group_id: 600_115,
    };
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response));
    let host = HostBuilder::new()
        .with_mock_client(mock)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(TorrentInjectionOptions {
            copy_torrent_to: Some(injection_path.clone()),
        })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();
    fs::remove_dir_all(&injection_path)?;

    // Act
    let result = command.execute(&manifest).await;

    // Assert
    assert!(
        result.is_ok(),
        "publish should continue when torrent injection fails"
    );
    let success = result.expect("checked");
    assert!(success.response.is_some(), "publish should still upload");
    assert!(
        !success.warnings.is_empty(),
        "publish should record injection warning"
    );
    Ok(())
}

#[tokio::test]
async fn publish_dry_run_does_not_stage_or_inject() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let source_dir = TempDirectory::create("publish_dry_run_does_not_stage_or_inject_source");
    let source_path = source_dir.to_path_buf();
    fs::write(source_path.join("01 Track.flac"), "source track")?;
    let manifest = PublishManifest {
        dry_run: true,
        ..create_new_group_manifest(source_path.clone())
    };
    manifest.validate()?;
    let content_dir = TempDirectory::create("publish_dry_run_does_not_stage_or_inject_content");
    let output_dir = TempDirectory::create("publish_dry_run_does_not_stage_or_inject_output");
    let injection_dir = TempDirectory::create("publish_dry_run_does_not_stage_or_inject_injection");
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_client(MockGazelleClient::new())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir.to_path_buf()],
            output: output_dir.to_path_buf(),
            ..SharedOptions::mock()
        })
        .with_options(TorrentInjectionOptions {
            copy_torrent_to: Some(injection_dir.to_path_buf()),
        })
        .with_options(PublishArg {
            publish_path: PathBuf::from("/tmp/unused.yml"),
        })
        .expect_build();
    let command = host.services.get_required::<PublishCommand>();

    // Act
    let result = command.execute(&manifest).await?;

    // Assert
    assert!(result.response.is_none(), "dry run should skip upload");
    let staged_path = content_dir.join(
        source_path
            .file_name()
            .expect("source path should have file name"),
    );
    assert!(source_path.is_dir(), "source should remain in place");
    assert!(
        !staged_path.exists(),
        "dry run should not create staged source directory"
    );
    let copied_torrents: Vec<_> = fs::read_dir(&*injection_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "torrent"))
        .collect();
    assert!(
        copied_torrents.is_empty(),
        "dry run should not inject torrents"
    );
    Ok(())
}

#[tokio::test]
async fn publish_generates_structured_release_description_for_new_group() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_path = SAMPLE_SOURCES_DIR.join(album.dir_name());

    // Act
    let description =
        PublishCommand::create_release_description(&source_path, "Release notes", "FLAC Lossless");

    // Assert
    assert!(
        description.contains("Published and uploaded with [url="),
        "release description should include tool/version header"
    );
    assert!(
        description.contains("Release notes"),
        "release description should include manifest notes"
    );
    assert!(
        description.contains("[pad=0|10|0|20]Source[/pad] FLAC Lossless"),
        "release description should include source format/bitrate"
    );
    assert!(
        description.contains("[pad=0|10|0|19]Details[/pad] [pre]"),
        "release description should include Details section"
    );
    assert!(
        description.contains("[pad=0|10|0|31]Tags[/pad] [hide][pre]"),
        "release description should include hidden Tags section"
    );
    Ok(())
}

fn create_new_group_manifest(source_path: PathBuf) -> PublishManifest {
    PublishManifest {
        source_path,
        torrent_path: None,
        manual_checks_ack: true,
        dry_run: false,
        mode: PublishMode::NewGroup,
        release_desc: "Release notes".to_owned(),
        new_group: Some(PublishNewGroup {
            title: "Album Title".to_owned(),
            year: 2024,
            release_type: 1,
            media: "WEB".to_owned(),
            tags: vec!["electronic".to_owned(), "ambient".to_owned()],
            album_desc: "Group description".to_owned(),
            request_id: Some(364_781),
            image: Some("https://example.com/cover.jpg".to_owned()),
            artists: vec![PublishArtist {
                name: "Artist Name".to_owned(),
                role: 1,
            }],
            edition: PublishNewGroupEdition {
                unknown_release: false,
                remaster: Some(true),
                year: 2024,
                title: "Digital".to_owned(),
                record_label: "Label".to_owned(),
                catalogue_number: "CAT-001".to_owned(),
                format: "FLAC".to_owned(),
                bitrate: "Lossless".to_owned(),
            },
        }),
        existing_group: None,
    }
}

fn create_existing_group_manifest(source_path: PathBuf) -> PublishManifest {
    PublishManifest {
        source_path,
        torrent_path: None,
        manual_checks_ack: true,
        dry_run: false,
        mode: PublishMode::ExistingGroup,
        release_desc: "Release notes".to_owned(),
        new_group: None,
        existing_group: Some(PublishExistingGroup {
            group_id: 123_456,
            remaster_year: 2024,
            remaster_title: "Digital".to_owned(),
            remaster_record_label: "Label".to_owned(),
            remaster_catalogue_number: "CAT-001".to_owned(),
            media: "WEB".to_owned(),
            format: "FLAC".to_owned(),
            bitrate: "Lossless".to_owned(),
        }),
    }
}
