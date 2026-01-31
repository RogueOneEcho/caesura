use crate::testing_prelude::*;
use gazelle_api::MockGazelleClient;
use std::fs;

/// Test that `UploadCommand` succeeds with a valid transcoded source.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_succeeds_with_valid_source() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let host = build_upload_test_host(&transcode, &test_dir).await;
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "upload should succeed");
    assert!(status.formats.is_some(), "should have format statuses");
    let formats = status.formats.as_ref().expect("checked above");
    assert!(!formats.is_empty(), "should have at least one format");

    Ok(())
}

/// Test that `UploadCommand` with `dry_run=true` does not call the API.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_dry_run_skips_api_call() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(transcode.album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            ..TargetOptions::default()
        })
        .with_options(UploadOptions {
            dry_run: true,
            ..UploadOptions::default()
        })
        .build();
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "dry run should succeed");
    assert!(
        status.formats.is_none(),
        "dry run should not record formats"
    );
    assert!(status.errors.is_none(), "dry run should have no errors");

    Ok(())
}

/// Test that `UploadCommand` fails validation with non-existent `copy_torrent_to` path.
#[tokio::test]
async fn upload_command_validation_failure_returns_false() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(UploadOptions {
            copy_torrent_to: Some(PathBuf::from("/nonexistent/path")),
            ..UploadOptions::default()
        })
        .with_options(SourceArg {
            source: Some(AlbumConfig::TORRENT_ID.to_string()),
        })
        .build();

    let command = host.services.get_required::<UploadCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(matches!(result, Ok(false)));

    Ok(())
}

/// Test that `UploadCommand` fails when the torrent file is missing.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_missing_torrent_fails() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();

    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .build();

    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should get source");

    let transcode_status = transcoder.execute(&source).await;
    assert!(transcode_status.success, "transcode should succeed");

    // Delete torrent files
    let output_dir = test_dir.output();
    delete_torrent_files(&output_dir);

    let command = host.services.get_required::<UploadCommand>();

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(!status.success, "upload should fail with missing torrent");
    assert!(status.errors.is_some(), "should have error messages");

    Ok(())
}

fn delete_torrent_files(dir: &PathBuf) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "torrent") {
                let _ = fs::remove_file(&path);
            }
            if path.is_dir() {
                delete_torrent_files(&path);
            }
        }
    }
}

/// Test that `UploadCommand` copies transcode to content directory when enabled.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_copies_to_content_dir() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let copy_target = TempDirectory::create("content_copy_target");
    let host = HostBuilder::new()
        .with_mock_api(transcode.album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            // First content dir is for copy destination, second is where source files are found
            content: vec![copy_target.clone(), SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            allow_existing: true, // Allow existing since we're using pre-generated transcodes
            ..TargetOptions::default()
        })
        .with_options(UploadOptions {
            copy_transcode_to_content_dir: true,
            ..UploadOptions::default()
        })
        .build();
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "upload should succeed");
    let entries: Vec<_> = fs::read_dir(&copy_target).expect("read dir").collect();
    assert!(!entries.is_empty(), "should have copied transcodes");

    Ok(())
}

/// Test that `UploadCommand` copies transcode to custom directory.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_copies_to_custom_dir() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let copy_target = TempDirectory::create("custom_copy_target");
    let host = HostBuilder::new()
        .with_mock_api(transcode.album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            ..TargetOptions::default()
        })
        .with_options(UploadOptions {
            copy_transcode_to: Some(copy_target.clone()),
            ..UploadOptions::default()
        })
        .build();
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "upload should succeed");
    let entries: Vec<_> = fs::read_dir(&copy_target).expect("read dir").collect();
    assert!(!entries.is_empty(), "should have copied transcodes");

    Ok(())
}

/// Test that `UploadCommand` copies torrent file to specified directory.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_copies_torrent_file() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let torrent_target = TempDirectory::create("torrent_copy_target");
    let host = HostBuilder::new()
        .with_mock_api(transcode.album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            ..TargetOptions::default()
        })
        .with_options(UploadOptions {
            copy_torrent_to: Some(torrent_target.clone()),
            ..UploadOptions::default()
        })
        .build();
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "upload should succeed");
    let torrent_files: Vec<_> = fs::read_dir(&torrent_target)
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "torrent"))
        .collect();
    assert!(!torrent_files.is_empty(), "should have copied torrents");

    Ok(())
}

/// Test that `UploadCommand` handles API failure gracefully.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_api_failure_sets_error() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();

    // Create a mock client that fails on upload
    let torrent_path = SAMPLE_SOURCES_DIR.join(transcode.album.torrent_filename());
    let torrent_bytes = fs::read(torrent_path).expect("torrent file should exist");

    let mock = MockGazelleClient::new()
        .with_get_torrent(Ok(gazelle_api::TorrentResponse {
            group: gazelle_api::Group {
                id: 123,
                name: transcode.album.album.to_owned(),
                year: transcode.album.year,
                category_name: "Music".to_owned(),
                music_info: Some(gazelle_api::Credits {
                    artists: vec![gazelle_api::Credit {
                        id: 1,
                        name: transcode.album.artist.to_owned(),
                    }],
                    ..gazelle_api::Credits::default()
                }),
                ..gazelle_api::Group::default()
            },
            torrent: gazelle_api::Torrent {
                id: AlbumConfig::TORRENT_ID,
                format: "FLAC".to_owned(),
                encoding: "Lossless".to_owned(),
                media: "WEB".to_owned(),
                ..gazelle_api::Torrent::default()
            },
        }))
        .with_get_torrent_group(Ok(gazelle_api::GroupResponse {
            group: gazelle_api::Group {
                id: 123,
                name: transcode.album.album.to_owned(),
                year: transcode.album.year,
                category_name: "Music".to_owned(),
                ..gazelle_api::Group::default()
            },
            torrents: vec![gazelle_api::Torrent {
                id: AlbumConfig::TORRENT_ID,
                format: "FLAC".to_owned(),
                encoding: "Lossless".to_owned(),
                media: "WEB".to_owned(),
                ..gazelle_api::Torrent::default()
            }],
        }))
        .with_download_torrent(Ok(torrent_bytes))
        .with_upload_torrent(Err(gazelle_api::GazelleError::Upload {
            error: "Upload failed".to_owned(),
        }));

    let mut builder = HostBuilder::new();

    #[allow(clippy::as_conversions)]
    let client: di::Ref<Box<dyn gazelle_api::GazelleClientTrait + Send + Sync>> =
        di::Ref::new(Box::new(mock) as Box<dyn gazelle_api::GazelleClientTrait + Send + Sync>);
    builder
        .services
        .add(di::singleton_as_self().from(move |_| client.clone()));

    let _ = builder
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            ..TargetOptions::default()
        });
    let host = builder.build();
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(!status.success, "upload should fail on API error");
    assert!(status.errors.is_some(), "should have errors");

    Ok(())
}

/// Test that `UploadCommand` captures torrent ID from API response.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_captures_response_ids() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let host = build_upload_test_host(&transcode, &test_dir).await;
    let (source, command) = get_source_and_command(&host).await;

    // Act
    let status = command.execute(&source).await;

    // Assert
    assert!(status.success, "upload should succeed");
    assert!(status.formats.is_some(), "should have format statuses");
    for format_status in status.formats.as_ref().expect("checked") {
        assert!(format_status.id > 0, "should have valid ID");
    }

    Ok(())
}

/// Test that upload succeeds when copy target already exists.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "FLAC output differs on ARM")]
async fn upload_command_skip_existing_copy_succeeds() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let transcode = TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
    let test_dir = TestDirectory::new();
    let host = build_upload_test_host(&transcode, &test_dir).await;
    let (source, command) = get_source_and_command(&host).await;

    // Act - run twice
    let status1 = command.execute(&source).await;
    assert!(status1.success, "first upload should succeed");

    let status2 = command.execute(&source).await;

    // Assert
    assert!(status2.success, "second upload should succeed");

    Ok(())
}

/// Helper to build a host configured for upload tests with pre-generated transcodes.
async fn build_upload_test_host(transcode: &TranscodeConfig, test_dir: &TestDirectory) -> Host {
    HostBuilder::new()
        .with_mock_api(transcode.album.clone())
        .with_test_options(test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: SAMPLE_TRANSCODES_DIR.clone(),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![transcode.target],
            ..TargetOptions::default()
        })
        .build()
}

/// Helper to get source and upload command from a host.
async fn get_source_and_command(host: &Host) -> (Source, di::Ref<UploadCommand>) {
    let provider = host.services.get_required::<SourceProvider>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should get source");
    let command = host.services.get_required::<UploadCommand>();
    (source, command)
}
