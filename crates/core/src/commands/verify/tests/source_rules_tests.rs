use crate::testing_prelude::*;

#[test]
fn source_issue_serialize() {
    // Arrange
    let rules = all_source_issues();

    // Act
    let yaml = yaml_to_string(&rules).expect("Failed to serialize SourceIssue");

    // Assert
    assert_snapshot!(yaml);
}

#[expect(
    deprecated,
    reason = "constructs deprecated variants for serialization test"
)]
fn all_source_issues() -> Vec<SourceIssue> {
    let mut existing_formats = BTreeSet::new();
    existing_formats.insert(ExistingFormat::_320);
    existing_formats.insert(ExistingFormat::Flac);
    let file = PathBuf::from("/path/to/file.flac");
    vec![
        SourceIssue::IdError {
            details: "invalid id".to_owned(),
        },
        SourceIssue::Id(IdProviderError::NoMatch),
        SourceIssue::GroupMismatch {
            actual: 1,
            expected: 2,
        },
        SourceIssue::ApiResponse {
            action: "get".to_owned(),
            status_code: 500,
            error: "server error".to_owned(),
        },
        SourceIssue::Provider,
        SourceIssue::Api {
            response: GazelleSerializableError::NotFound {
                message: "not found".to_owned(),
            },
        },
        SourceIssue::NotFound,
        SourceIssue::Category {
            actual: "Music".to_owned(),
        },
        SourceIssue::Scene,
        SourceIssue::LossyMaster,
        SourceIssue::LossyWeb,
        SourceIssue::Trumpable,
        SourceIssue::Unconfirmed,
        SourceIssue::Excluded {
            tags: vec!["tag1".to_owned()],
        },
        SourceIssue::Existing {
            formats: existing_formats,
        },
        SourceIssue::NotSource {
            format: "MP3".to_owned(),
            encoding: "320".to_owned(),
        },
        SourceIssue::MissingDirectory {
            path: PathBuf::from("/path/to/source"),
        },
        SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("subdir"),
        },
        SourceIssue::NoFlacs {
            path: PathBuf::from("/path/to/source"),
        },
        SourceIssue::FlacCount {
            expected: 10,
            actual: 8,
        },
        SourceIssue::Imdl {
            details: "\u{1b}[2m[1/2]\u{1b}[0m 💾 \u{1b}[1mLoading metainfo from `./cache/torrents/123456.red.torrent`…\u{1b}[0m\n\u{1b}[2m[2/2]\u{1b}[0m 🧮 \u{1b}[1mVerifying pieces from `/srv/shared/music/`…\u{1b}[0m\nPieces corrupted.\n\u{1b}[1;31merror\u{1b}[0m\u{1b}[1m: Torrent verification failed.\u{1b}[0m\n".to_owned(),
        },
        SourceIssue::HashCheck { piece_index: 42 },
        SourceIssue::MissingFile { path: file.clone() },
        SourceIssue::OpenFile {
            path: file.clone(),
            error: "permission denied".to_owned(),
        },
        SourceIssue::ExcessContent,
        SourceIssue::Length {
            path: PathBuf::from("/path/to/file"),
            excess: 10,
        },
        SourceIssue::MissingTags {
            path: file.clone(),
            tags: vec!["Title".to_owned(), "Artist".to_owned()],
        },
        SourceIssue::FlacError {
            path: file.clone(),
            error: "I/O Error".to_owned(),
        },
        SourceIssue::SampleRate {
            path: file.clone(),
            rate: 44100,
        },
        SourceIssue::BitRate {
            path: file.clone(),
            rate: 128,
        },
        SourceIssue::Duration {
            path: file.clone(),
            seconds: 43200,
        },
        SourceIssue::Channels {
            path: file.clone(),
            count: 2,
        },
        SourceIssue::Error {
            domain: "Torrent".to_owned(),
            details: "something went wrong".to_owned(),
        },
    ]
}
