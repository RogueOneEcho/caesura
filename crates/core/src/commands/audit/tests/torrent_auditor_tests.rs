use crate::testing_prelude::*;

#[test]
fn torrent_auditor_execute_clean() {
    // Arrange
    let bytes = TorrentBuilder::new().with_multi_file(["song.flac"]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(output.name, Some("album".to_owned()));
    assert!(output.issues.is_none());
}

/// A non-UTF-8 path element yields a Windows-1252 suggestion recovering the accented character.
#[test]
fn torrent_auditor_execute_non_utf8() {
    // Arrange
    let component = splice(b"song", E_ACUTE, b".flac");
    let bytes = TorrentBuilder::new().with_multi_file([component]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    let issue = output
        .issues
        .iter()
        .flatten()
        .find(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::NonUtf8))
        .expect("expected NonUtf8");
    let suggestions = issue.suggestions.as_ref().expect("expected suggestions");
    assert!(
        suggestions
            .iter()
            .any(|suggestion| suggestion.value == "song\u{e9}.flac"),
        "expected a windows-1252 suggestion, got: {suggestions:?}"
    );
}

#[test]
fn torrent_auditor_execute_zero_width_space() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song\u{200B}.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        output
            .issues
            .iter()
            .flatten()
            .any(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::InvisibleChars)),
        "expected InvisibleChars, got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_rtl_override() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song\u{202E}.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        output
            .issues
            .iter()
            .flatten()
            .any(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::LibtorrentStripped)),
        "expected LibtorrentStripped, got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_traversal() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["..", "song.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        output
            .issues
            .iter()
            .flatten()
            .any(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::UnsafeSegment)),
        "expected UnsafeSegment, got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_forward_slash() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song/evil.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([
            AuditIssueKind::Path(AuditPathIssueKind::RestrictedChars),
            AuditIssueKind::Path(AuditPathIssueKind::UnsafeSegment),
            AuditIssueKind::Path(AuditPathIssueKind::LibtorrentStripped),
        ]),
    );
}

#[test]
fn torrent_auditor_execute_trailing_slash() {
    // Arrange
    let bytes = TorrentBuilder::new().with_multi_file(["song/"]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([
            AuditIssueKind::Path(AuditPathIssueKind::UnsafeSegment),
            AuditIssueKind::Path(AuditPathIssueKind::RestrictedChars),
            AuditIssueKind::Path(AuditPathIssueKind::LibtorrentStripped),
        ]),
    );
}

#[test]
fn torrent_auditor_execute_backslash() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song\\evil.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([
            AuditIssueKind::Path(AuditPathIssueKind::RestrictedChars),
            AuditIssueKind::Path(AuditPathIssueKind::UnsafeSegment),
            AuditIssueKind::Path(AuditPathIssueKind::LibtorrentStripped),
        ]),
    );
}

#[test]
fn torrent_auditor_execute_decomposed() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["cafe\u{0301}.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::Path(AuditPathIssueKind::Decomposed)]),
    );
}

#[test]
fn torrent_auditor_execute_composed() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["caf\u{00e9}.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(output.issues.is_none(), "got: {:?}", output.issues);
}

#[test]
fn torrent_auditor_execute_ignore_invisible() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song\u{200B}.flac"])
        .build();
    let auditor = TorrentAuditor::new(AuditOptions {
        ignore_invisible: true,
        ..AuditOptions::default()
    });

    // Act
    let output = auditor.execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::InvisibleChars),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_ignore_libtorrent() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["song\u{202E}.flac"])
        .build();
    let auditor = TorrentAuditor::new(AuditOptions {
        ignore_libtorrent: true,
        ..AuditOptions::default()
    });

    // Act
    let output = auditor.execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::LibtorrentStripped),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_ignore_unsafe() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["..", "song.flac"])
        .build();
    let auditor = TorrentAuditor::new(AuditOptions {
        ignore_unsafe: true,
        ..AuditOptions::default()
    });

    // Act
    let output = auditor.execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::UnsafeSegment),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_ignore_nfd() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_multi_file(["cafe\u{0301}.flac"])
        .build();
    let auditor = TorrentAuditor::new(AuditOptions {
        ignore_nfd: true,
        ..AuditOptions::default()
    });

    // Act
    let output = auditor.execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::Decomposed),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_name_non_utf8() {
    // Arrange
    let name = splice(b"album", E_ACUTE, b"");
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(["song.flac"])])
                .with_string("name", name),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        output
            .issues
            .iter()
            .flatten()
            .any(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::NonUtf8)),
        "expected NonUtf8, got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_single_file() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_integer("length", 10)
                .with_string("name", "song.flac"),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    let issues = output.issues.expect("expected issues");
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == AuditIssueKind::NoFiles),
        "expected NoFiles, got: {issues:?}"
    );
}

#[test]
fn torrent_auditor_execute_not_bencode() {
    // Arrange
    let bytes = b"this is not bencode".to_vec();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    let issues = output.issues.expect("expected issues");
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == AuditIssueKind::Parse)
    );
}

/// A CESU-8 path element yields a first `Cesu8` suggestion recovering the emoji.
#[test]
fn torrent_auditor_execute_cesu8() {
    // Arrange
    let mut component = b"track ".to_vec();
    component.extend_from_slice(&[0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x89]);
    component.push(b' ');
    component.extend_from_slice(&[0xED, 0xA0, 0xBE, 0xED, 0xB5, 0xB4]);
    component.extend_from_slice(b".flac");
    let bytes = TorrentBuilder::new().with_multi_file([component]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::Path(AuditPathIssueKind::NonUtf8)]),
    );
    let issue = output
        .issues
        .iter()
        .flatten()
        .find(|issue| issue.kind == AuditIssueKind::Path(AuditPathIssueKind::NonUtf8))
        .expect("expected NonUtf8");
    let suggestions = issue.suggestions.as_ref().expect("expected suggestions");
    let first = suggestions.first().expect("expected a suggestion");
    assert_eq!(first.kind, AuditSuggestionKind::Cesu8);
    assert_eq!(first.value, "track \u{1F609} \u{1F974}.flac");
}

#[test]
fn torrent_auditor_execute_comment() {
    // Arrange
    let url = "https://example.invalid/torrents.php?id=1&torrentid=2#torrent2";
    let bytes = TorrentBuilder::new()
        .with_string("comment", url)
        .with_multi_file(["song.flac"])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(output.id, Some(2));
    assert_eq!(output.url.as_deref(), Some(url));
}

#[test]
fn torrent_auditor_execute_lost_extension() {
    // Arrange
    let component = splice(b"song", I_ACUTE, b".flac");
    let bytes = TorrentBuilder::new().with_multi_file([component]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([
            AuditIssueKind::Path(AuditPathIssueKind::BrokenExtension),
            AuditIssueKind::Path(AuditPathIssueKind::NonUtf8),
        ]),
    );
}

#[test]
fn torrent_auditor_execute_lost_extension_midname() {
    // Arrange
    let component = splice(b"so", I_ACUTE, b"ng.flac");
    let bytes = TorrentBuilder::new().with_multi_file([component]).build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::BrokenExtension),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_lost_extension_directory() {
    // Arrange
    let directory = splice(b"dir", I_ACUTE, b"");
    let bytes = TorrentBuilder::new()
        .with_multi_file([directory, b"song.flac".to_vec()])
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::BrokenExtension),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_ignore_lost_extension() {
    // Arrange
    let component = splice(b"song", I_ACUTE, b".flac");
    let bytes = TorrentBuilder::new().with_multi_file([component]).build();
    let auditor = TorrentAuditor::new(AuditOptions {
        ignore_lost_extension: true,
        ..AuditOptions::default()
    });

    // Act
    let output = auditor.execute_bytes(&bytes);

    // Assert
    assert!(
        !output.has_path_kind(AuditPathIssueKind::BrokenExtension),
        "got: {:?}",
        output.issues
    );
}

#[test]
fn torrent_auditor_execute_name_utf8() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(["song.flac"])])
                .with_string("name", "legacy")
                .with_string("name.utf-8", "chosen"),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(output.name, Some("chosen".to_owned()));
}

#[test]
fn torrent_auditor_execute_path_utf8() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries(
                    "files",
                    vec![
                        TorrentBuilder::file(["song.flac"])
                            .with_list("path.utf-8", ["song\u{200B}.flac"]),
                    ],
                )
                .with_string("name", "album"),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::Path(AuditPathIssueKind::InvisibleChars)]),
    );
}

#[test]
fn torrent_auditor_execute_name_utf8_wrong_type() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(["song.flac"])])
                .with_string("name", "album")
                .with_integer("name.utf-8", 5),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::NameDivergence]),
    );
}

#[test]
fn torrent_auditor_execute_path_utf8_wrong_type() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries(
                    "files",
                    vec![
                        TorrentBuilder::file(["song.flac"]).with_string("path.utf-8", "song.flac"),
                    ],
                )
                .with_string("name", "album"),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::PathDivergence]),
    );
}

#[test]
fn torrent_auditor_execute_name_utf8_empty() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(["song.flac"])])
                .with_string("name", "album")
                .with_string("name.utf-8", ""),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::NameEmpty]),
    );
}

#[test]
fn torrent_auditor_execute_path_utf8_empty() {
    // Arrange
    let bytes = TorrentBuilder::new()
        .with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries(
                    "files",
                    vec![
                        TorrentBuilder::file(["song.flac"])
                            .with_list("path.utf-8", Vec::<RawString>::new()),
                    ],
                )
                .with_string("name", "album"),
        )
        .build();

    // Act
    let output = TorrentAuditor::mock().execute_bytes(&bytes);

    // Assert
    assert_eq!(
        output.get_issue_kinds(),
        HashSet::from([AuditIssueKind::PathEmpty]),
    );
}
