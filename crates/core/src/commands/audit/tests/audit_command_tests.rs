use crate::testing_prelude::*;

/// A problematic torrent is detected.
#[test]
fn audit_command_reports_problematic() {
    // Arrange
    init_logger();
    let dir = TempDirectory::create("audit_command_reports_problematic");
    let torrents = dir.join("torrents");
    create_dir_all(&torrents).expect("create torrents dir");
    let component = splice(b"song", E_ACUTE, b".flac");
    let torrent = TorrentBuilder::new().with_multi_file([component]).build();
    write(torrents.join("bad.torrent"), torrent).expect("write torrent");

    // Act
    let summary = AuditCommand::mock()
        .execute(&torrents)
        .expect("should scan");

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
