use crate::testing_prelude::*;

/// A problematic torrent in a scanned directory is detected.
#[tokio::test]
async fn audit_command_execute_reports_problematic() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let dir = TempDirectory::create("audit_command_execute_reports_problematic");
    let torrents = dir.join("torrents");
    create_dir_all(&torrents).expect("create torrents dir");
    let component = splice(b"song", E_ACUTE, b".flac");
    let torrent = TorrentBuilder::new().with_multi_file([component]).build();
    write(torrents.join("bad.torrent"), torrent).expect("write torrent");
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let command = host.services.get_required::<AuditCommand>();

    // Act
    let summary = command.execute(&[torrents.join("bad.torrent")]);

    // Assert
    assert_eq!(summary.total, 1);
    assert_eq!(summary.issues.len(), 1);
    let item = summary.issues.first().expect("expected an item");
    let issues = item.issues.as_ref().expect("expected issues");
    let issue = issues.first().expect("expected an issue");
    assert_eq!(
        issue.kind,
        AuditIssueKind::Path(AuditPathIssueKind::NonUtf8)
    );
}

/// Kinds are counted per torrent, sorted by count then name, and shown with percentages.
#[test]
fn audit_summary_kind_table() {
    // Arrange
    let summary = AuditSummary {
        total: 5,
        issues: vec![
            AuditItem {
                issues: Some(vec![
                    AuditIssue::from(AuditIssueKind::Path(AuditPathIssueKind::NonUtf8)),
                    AuditIssue::from(AuditIssueKind::Path(AuditPathIssueKind::NonUtf8)),
                ]),
                ..AuditItem::default()
            },
            AuditItem {
                issues: Some(vec![AuditIssue::from(AuditIssueKind::Path(
                    AuditPathIssueKind::NonUtf8,
                ))]),
                ..AuditItem::default()
            },
            AuditItem {
                issues: Some(vec![AuditIssue::from(AuditIssueKind::Parse)]),
                ..AuditItem::default()
            },
            AuditItem {
                issues: Some(vec![AuditIssue::from(AuditIssueKind::Path(
                    AuditPathIssueKind::LibtorrentStripped,
                ))]),
                ..AuditItem::default()
            },
        ],
    };

    // Act
    let output = summary.kind_table();

    // Assert
    assert_snapshot!(output);
}

/// A problematic torrent downloaded by id is detected.
#[tokio::test]
async fn audit_command_execute_cli_id_reports_problematic() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let component = splice(b"song", E_ACUTE, b".flac");
    let torrent = TorrentBuilder::new().with_multi_file([component]).build();
    let api = MockGazelleClient::new().with_download_torrent(Ok(torrent));
    let host = HostBuilder::new()
        .with_mock_client(api)
        .with_test_options(&test_dir)
        .await
        .with_options(AuditArgs {
            audit_arg: "12345".to_owned(),
        })
        .expect_build();
    let command = host.services.get_required::<AuditCommand>();

    // Act
    let is_clean = command.execute_cli().await.expect("audit should run");

    // Assert
    assert!(!is_clean, "problematic torrent should report issues");
}

/// The id flow fails validation before any download when credentials are missing.
#[tokio::test]
async fn audit_command_execute_cli_id_missing_credentials() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let api = MockGazelleClient::new();
    let host = HostBuilder::new()
        .with_mock_client(api)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            api_key: String::new(),
            ..SharedOptions::mock()
        })
        .with_options(AuditArgs {
            audit_arg: "12345".to_owned(),
        })
        .expect_build();
    let command = host.services.get_required::<AuditCommand>();

    // Act
    let error = command
        .execute_cli()
        .await
        .expect_err("should fail validation");

    // Assert
    assert_eq!(error.action(), &AuditAction::ValidateOptions);
}

/// A clean torrent downloaded by id reports no issues.
#[tokio::test]
async fn audit_command_execute_cli_id_clean() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let torrent = TorrentBuilder::new()
        .with_multi_file([b"song.flac".to_vec()])
        .build();
    let api = MockGazelleClient::new().with_download_torrent(Ok(torrent));
    let host = HostBuilder::new()
        .with_mock_client(api)
        .with_test_options(&test_dir)
        .await
        .with_options(AuditArgs {
            audit_arg: "12345".to_owned(),
        })
        .expect_build();
    let command = host.services.get_required::<AuditCommand>();

    // Act
    let is_clean = command.execute_cli().await.expect("audit should run");

    // Assert
    assert!(is_clean, "clean torrent should report no issues");
}

/// A non-numeric input routes through the offline path scan.
#[tokio::test]
async fn audit_command_execute_cli_path() {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let dir = TempDirectory::create("audit_command_execute_cli_path");
    let torrents = dir.join("torrents");
    create_dir_all(&torrents).expect("create torrents dir");
    let component = splice(b"song", E_ACUTE, b".flac");
    let torrent = TorrentBuilder::new().with_multi_file([component]).build();
    write(torrents.join("bad.torrent"), torrent).expect("write torrent");
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .with_options(AuditArgs {
            audit_arg: torrents.to_string_lossy().into_owned(),
        })
        .expect_build();
    let command = host.services.get_required::<AuditCommand>();

    // Act
    let is_clean = command.execute_cli().await.expect("audit should run");

    // Assert
    assert!(!is_clean, "problematic torrent should report issues");
}
