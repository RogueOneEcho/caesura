use crate::testing_prelude::*;

fn reportable_issue() -> SourceIssue {
    SourceIssue::NoTags {
        path: PathBuf::from("/content/Artist - Album [2024]/02 - Track.flac"),
    }
}

#[tokio::test]
async fn source_reporter_execute_no_reportable_issues() {
    // Arrange
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![SourceIssue::NotFound];
    let reports_dir = test_dir.reports();

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
}

#[tokio::test]
async fn source_reporter_execute_reports_disabled() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .with_options(ReportOptions {
            reports_dir: reports_dir.clone(),
            no_reports: true,
        })
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![reportable_issue()];

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
}

#[tokio::test]
async fn source_reporter_execute_no_tags_issue() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![reportable_issue()];
    let expected = reports_dir.join(format!("red-{}.md", source.torrent.id));

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(expected.exists(), "expected file {expected:?}");
    let contents = read_to_string(&expected).expect("read report file");
    assert!(contents.contains("# "), "should have title header");
    assert!(contents.contains("No tags"), "should mention no tags");
}

#[tokio::test]
async fn source_reporter_execute_existing_report() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let target = reports_dir.join(format!("red-{}.md", source.torrent.id));

    // Act
    reporter
        .execute(&source, &[reportable_issue()])
        .expect("first execute should succeed");
    let first = read_to_string(&target).expect("read first report");
    reporter
        .execute(
            &source,
            &[SourceIssue::MissingTags {
                path: PathBuf::from("/content/Artist - Album [2024]/02 - Track.flac"),
                tags: vec!["composer".to_owned()],
            }],
        )
        .expect("second execute should succeed");
    let second = read_to_string(&target).expect("read second report");

    // Assert
    assert_ne!(first, second, "file should have been overwritten");
    assert!(second.contains("Missing tags: composer"));
}

#[tokio::test]
async fn source_reporter_execute_missing_dir() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.join("nested").join("reports");
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .with_options(ReportOptions {
            reports_dir: reports_dir.clone(),
            no_reports: false,
        })
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();

    // Act
    reporter
        .execute(&source, &[reportable_issue()])
        .expect("execute should succeed");

    // Assert
    assert!(reports_dir.exists(), "expected reports dir to be created");
    let files: Vec<_> = reports_dir.read_dir().expect("read reports dir").collect();
    assert_eq!(files.len(), 1, "expected exactly one report file");
}

#[tokio::test]
async fn source_reporter_execute_blocked_by_hash_check() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![
        reportable_issue(),
        SourceIssue::HashCheck { piece_index: 0 },
    ];

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
}

#[tokio::test]
async fn source_reporter_execute_blocked_by_trumpable() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![reportable_issue(), SourceIssue::Trumpable];

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
}

#[tokio::test]
async fn source_reporter_execute_blocked_by_hash_check_skipped() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .with_options(VerifyOptions {
            no_hash_check: true,
            exclude_tags: None,
        })
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![reportable_issue()];

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
}

#[tokio::test]
async fn source_reporter_execute_not_blocked_by_scene() {
    // Arrange
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let host = HostBuilder::new()
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let reporter = host.services.get_required::<SourceReporter>();
    let source = Source::mock();
    let issues = vec![reportable_issue(), SourceIssue::Scene];
    let expected = reports_dir.join(format!("red-{}.md", source.torrent.id));

    // Act
    reporter
        .execute(&source, &issues)
        .expect("execute should succeed");

    // Assert
    assert!(expected.exists(), "expected report file {expected:?}");
}
