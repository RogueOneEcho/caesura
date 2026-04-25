use crate::testing_prelude::*;

/// Verify `cross` errors when validation fails (no cross config).
#[tokio::test]
async fn cross_command_validation_fails_without_cross_config() {
    // Arrange
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_client(MockGazelleClient::default())
        .with_mock_torrent_client(MockQBittorrentClient::default())
        .with_test_options(&test_dir)
        .await
        .with_options(QbitOptions::mock())
        .with_options(SourceArg {
            source: AlbumConfig::TORRENT_ID.to_string(),
        })
        .expect_build();
    let command = host.services.get_required::<CrossCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(
        result.is_err(),
        "should fail validation without cross config"
    );
}

/// Verify `cross` returns `Ok(false)` when the source can't be resolved.
#[tokio::test]
async fn cross_command_unresolvable_source_returns_false() {
    // Arrange
    let test_dir = TestDirectory::new();
    let cross_config = make_cross_config(&test_dir, "OPS").await;
    let main_client = MockGazelleClient::default().with_get_torrent(Err(GazelleError {
        operation: GazelleOperation::ApiResponse(ApiResponseKind::NotFound),
        source: ErrorSource::ApiResponse(ApiResponseError {
            message: "not found".to_owned(),
            status: 404,
        }),
    }));
    let host = HostBuilder::new()
        .with_mock_client(main_client)
        .with_mock_cross_client(MockGazelleClient::default())
        .with_mock_torrent_client(MockQBittorrentClient::default())
        .with_test_options(&test_dir)
        .await
        .with_options(QbitOptions::mock())
        .with_options(QbitCrossOptions::mock())
        .with_options(CrossConfigOptions {
            cross_config: Some(cross_config),
        })
        .with_options(CrossOptions {
            dry_run: true,
            copy_cross_torrent_to: None,
        })
        .with_options(SourceArg {
            source: "999999999".to_owned(),
        })
        .expect_build();
    let command = host.services.get_required::<CrossCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert_eq!(result.ok(), Some(false));
}

/// Verify `cross` errors when neither inject nor copy target is configured and `dry_run` is false.
#[tokio::test]
async fn cross_command_validation_fails_without_inject_copy_or_dry_run() {
    // Arrange
    let test_dir = TestDirectory::new();
    let cross_config = make_cross_config(&test_dir, "OPS").await;
    let host = HostBuilder::new()
        .with_mock_client(MockGazelleClient::default())
        .with_mock_cross_client(MockGazelleClient::default())
        .with_mock_torrent_client(MockQBittorrentClient::default())
        .with_test_options(&test_dir)
        .await
        .with_options(QbitOptions::mock())
        .with_options(CrossConfigOptions {
            cross_config: Some(cross_config),
        })
        .with_options(CrossOptions {
            dry_run: false,
            copy_cross_torrent_to: None,
        })
        .with_options(SourceArg {
            source: AlbumConfig::TORRENT_ID.to_string(),
        })
        .expect_build();
    let command = host.services.get_required::<CrossCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(
        result.is_err(),
        "should fail validation without qbit_cross, copy_cross_torrent_to, or dry_run"
    );
}

/// Build a minimal cross-indexer config file pointing at the given indexer.
async fn make_cross_config(test_dir: &TestDirectory, indexer: &str) -> PathBuf {
    let path = test_dir.cache().join("cross.yml");
    tokio_create_dir_all(&test_dir.cache())
        .await
        .expect("create cache dir");
    let yaml = format!(
        "indexer: {indexer}\nindexer_url: https://example.com\napi_key: test_key\nannounce_url: https://example.com/announce\n",
    );
    write(&path, yaml).expect("write cross config");
    path
}
