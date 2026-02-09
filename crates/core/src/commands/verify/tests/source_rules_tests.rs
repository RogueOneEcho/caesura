use crate::testing_prelude::*;
use crate::utils::SourceIssue::*;
use insta::assert_snapshot;

#[test]
fn test_serialize_source_rules_vec() {
    // Arrange
    let rules = all_source_issues();

    // Act
    let yaml = serde_yaml::to_string(&rules).expect("Failed to serialize SourceIssue");

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
        IdError {
            details: "invalid id".to_owned(),
        },
        Id(IdProviderError::NoMatch),
        GroupMismatch {
            actual: 1,
            expected: 2,
        },
        ApiResponse {
            action: "get".to_owned(),
            status_code: 500,
            error: "server error".to_owned(),
        },
        Provider,
        Api {
            response: gazelle_api::GazelleSerializableError::NotFound {
                message: "not found".to_owned(),
            },
        },
        NotFound,
        Category {
            actual: "Music".to_owned(),
        },
        Scene,
        LossyMaster,
        LossyWeb,
        Trumpable,
        Unconfirmed,
        Excluded {
            tags: vec!["tag1".to_owned()],
        },
        Existing {
            formats: existing_formats,
        },
        NotSource {
            format: "MP3".to_owned(),
            encoding: "320".to_owned(),
        },
        MissingDirectory {
            path: PathBuf::from("/path/to/source"),
        },
        UnnecessaryDirectory {
            prefix: PathBuf::from("subdir"),
        },
        NoFlacs {
            path: PathBuf::from("/path/to/source"),
        },
        FlacCount {
            expected: 10,
            actual: 8,
        },
        Imdl {
            details: "\u{1b}[2m[1/2]\u{1b}[0m 💾 \u{1b}[1mLoading metainfo from `./cache/torrents/123456.red.torrent`…\u{1b}[0m\n\u{1b}[2m[2/2]\u{1b}[0m 🧮 \u{1b}[1mVerifying pieces from `/srv/shared/music/`…\u{1b}[0m\nPieces corrupted.\n\u{1b}[1;31merror\u{1b}[0m\u{1b}[1m: Torrent verification failed.\u{1b}[0m\n".to_owned(),
        },
        HashCheck { piece_index: 42 },
        MissingFile { path: file.clone() },
        OpenFile {
            path: file.clone(),
            error: "permission denied".to_owned(),
        },
        ExcessContent,
        Length {
            path: PathBuf::from("/path/to/file"),
            excess: 10,
        },
        MissingTags {
            path: file.clone(),
            tags: vec!["Title".to_owned(), "Artist".to_owned()],
        },
        FlacError {
            path: file.clone(),
            error: "I/O Error".to_owned(),
        },
        SampleRate {
            path: file.clone(),
            rate: 44100,
        },
        BitRate {
            path: file.clone(),
            rate: 128,
        },
        Duration {
            path: file.clone(),
            seconds: 43200,
        },
        Channels {
            path: file.clone(),
            count: 2,
        },
        Error {
            domain: "Torrent".to_owned(),
            details: "something went wrong".to_owned(),
        },
    ]
}
