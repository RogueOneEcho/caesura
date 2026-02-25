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
        .with_options(PublishArg {
            publish_path: publish_path.clone(),
        })
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
        .with_options(PublishArg {
            publish_path: publish_path.clone(),
        })
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
    let mock = MockGazelleClient::new().with_upload_new_source(Ok(response.clone()));
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
    assert_eq!(actual.group_id, response.group_id);
    assert_eq!(actual.torrent_id, response.torrent_id);
    assert_eq!(
        result.permalink,
        Some(get_permalink(
            &"https://redacted.sh".to_owned(),
            response.group_id,
            response.torrent_id
        ))
    );
    assert_eq!(
        result.next_transcode,
        Some(format!("caesura transcode {}", response.torrent_id))
    );
    assert_eq!(
        result.next_upload,
        Some(format!("caesura upload {}", response.torrent_id))
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
        .with_upload_torrent(Ok(response.clone()));
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
    assert_eq!(actual.group_id, response.group_id);
    assert_eq!(actual.torrent_id, response.torrent_id);
    assert_eq!(
        result.permalink,
        Some(get_permalink(
            &"https://redacted.sh".to_owned(),
            response.group_id,
            response.torrent_id
        ))
    );
    assert_eq!(
        result.next_transcode,
        Some(format!("caesura transcode {}", response.torrent_id))
    );
    assert_eq!(
        result.next_upload,
        Some(format!("caesura upload {}", response.torrent_id))
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
