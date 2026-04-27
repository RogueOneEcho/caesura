#[allow(deprecated)]
use crate::testing_prelude::*;
use gazelle_api::GazelleSerializableError::*;

#[test]
#[allow(deprecated)]
#[expect(
    clippy::too_many_lines,
    reason = "flat enumeration of variants and expected YAML"
)]
fn source_issue_serialization() -> Result<(), YamlError> {
    // Arrange
    let example = vec![
        SourceIssue::IdError {
            details: "Hello, world!".to_owned(),
        },
        SourceIssue::Id(IdProviderError::NoMatch),
        SourceIssue::Id(IdProviderError::UrlInvalid),
        SourceIssue::Id(IdProviderError::TorrentFileSource {
            actual: "ABC".to_owned(),
            expected: "CBA".to_owned(),
        }),
        SourceIssue::ApiResponse {
            action: "test".to_owned(),
            status_code: 200,
            error: "test".to_owned(),
        },
        SourceIssue::Provider,
        SourceIssue::Api {
            response: BadRequest {
                message: String::new(),
            },
        },
        SourceIssue::Api {
            response: NotFound {
                message: String::new(),
            },
        },
        SourceIssue::Api {
            response: Deserialization {
                error: "A deserialization error occurred".to_owned(),
            },
        },
        SourceIssue::Api {
            response: Request {
                error: "A request error occurred".to_owned(),
            },
        },
        SourceIssue::Api {
            response: Other {
                status: 503,
                message: Some("Service unavailable".to_owned()),
            },
        },
        SourceIssue::Api {
            response: Other {
                status: 500,
                message: None,
            },
        },
        SourceIssue::NoDirectory,
        SourceIssue::InvalidFilePath {
            path: "./bad".to_owned(),
        },
    ];
    let expected = "- type: id_error
  details: Hello, world!
- type: id
  no_match: null
- type: id
  url_invalid: null
- type: id
  torrent_file_source:
    actual: ABC
    expected: CBA
- type: api_response
  action: test
  status_code: 200
  error: test
- type: provider
- type: api
  response:
    type: bad_request
    message: ''
- type: api
  response:
    type: not_found
    message: ''
- type: api
  response:
    type: deserialization
    error: A deserialization error occurred
- type: api
  response:
    type: request
    error: A request error occurred
- type: api
  response:
    type: other
    status: 503
    message: Service unavailable
- type: api
  response:
    type: other
    status: 500
    message: null
- type: no_directory
- type: invalid_file_path
  path: ./bad
";

    // Act
    let yaml = yaml_to_string(&example)?;
    println!("{yaml}");
    let deserialized: Vec<SourceIssue> = yaml_from_str(expected)?;

    // Assert
    assert_eq!(yaml, expected);
    assert_eq!(deserialized.len(), example.len());
    Ok(())
}

#[test]
#[allow(deprecated, clippy::similar_names)]
fn source_issue_provider_deprecated() -> Result<(), YamlError> {
    // Arrange
    let example = vec![
        SourceIssue::Api {
            response: BadRequest {
                message: String::new(),
            },
        },
        SourceIssue::Api {
            response: NotFound {
                message: String::new(),
            },
        },
        SourceIssue::Api {
            response: Deserialization {
                error: "A deserialization error occurred".to_owned(),
            },
        },
        SourceIssue::Api {
            response: Request {
                error: "A request error occurred".to_owned(),
            },
        },
        SourceIssue::Api {
            response: Other {
                status: 503,
                message: Some("Service unavailable".to_owned()),
            },
        },
        SourceIssue::Api {
            response: Other {
                status: 500,
                message: None,
            },
        },
    ];
    let before = "- type: provider
  BadRequest: null
- type: provider
  NotFound: null
- type: provider
  Deserialization: A deserialization error occurred
- type: provider
  Request: A request error occurred
- type: provider
  Unexpected:
  - 503
  - Service unavailable
- type: provider
  Empty: 500
";
    let after = "- type: api
  response:
    type: bad_request
    message: ''
- type: api
  response:
    type: not_found
    message: ''
- type: api
  response:
    type: deserialization
    error: A deserialization error occurred
- type: api
  response:
    type: request
    error: A request error occurred
- type: api
  response:
    type: other
    status: 503
    message: Service unavailable
- type: api
  response:
    type: other
    status: 500
    message: null
";

    // Act
    let before_deserialized: Vec<SourceIssue> = yaml_from_str(before)?;
    let after_deserialized: Vec<SourceIssue> = yaml_from_str(after)?;
    let yaml = yaml_to_string(&example)?;
    println!("{yaml}");
    let before_reserialized = yaml_to_string(&before_deserialized)?;
    println!("--------------------");
    println!("{before_reserialized}");

    // Assert
    assert_eq!(yaml, after);
    assert_eq!(before_deserialized.len(), after_deserialized.len());
    Ok(())
}

#[test]
fn source_issue_is_reportable() {
    let reportable = [
        SourceIssue::NoTags {
            path: PathBuf::from("/a.flac"),
        },
        SourceIssue::MissingTags {
            path: PathBuf::from("/b.flac"),
            tags: vec!["composer".to_owned()],
        },
        SourceIssue::FlacError {
            path: PathBuf::from("/c.flac"),
            error: "decode".to_owned(),
        },
        SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("CD1"),
        },
        SourceIssue::SampleRate {
            path: PathBuf::from("/d.flac"),
            rate: 192_000,
        },
    ];
    let not_reportable = [
        SourceIssue::NotFound,
        SourceIssue::Scene,
        SourceIssue::Trumpable,
        SourceIssue::MissingDirectory {
            path: PathBuf::from("/x"),
        },
        SourceIssue::NoDirectory,
        SourceIssue::InvalidFilePath {
            path: "./bad".to_owned(),
        },
    ];
    for issue in reportable {
        assert!(issue.is_reportable(), "expected reportable: {issue:?}");
    }
    for issue in not_reportable {
        assert!(!issue.is_reportable(), "expected not reportable: {issue:?}");
    }
}

#[test]
fn source_issue_report_type() {
    let issues = report_sample_issues();
    let output: Vec<_> = issues.iter().map(SourceIssue::report_type).collect();
    assert_yaml_snapshot!(output);
}

#[test]
fn source_issue_report_label() {
    let issues = report_sample_issues();
    let output: Vec<_> = issues.iter().map(SourceIssue::report_label).collect();
    assert_yaml_snapshot!(output);
}

#[test]
fn source_issue_affected_paths() {
    // Arrange
    let path = PathBuf::from("/a.flac");
    let issue = SourceIssue::NoTags { path: path.clone() };

    // Act
    let output = issue.affected_paths();

    // Assert
    assert_eq!(output, vec![path.as_path()]);
}

fn report_sample_issues() -> Vec<SourceIssue> {
    vec![
        SourceIssue::NoTags {
            path: PathBuf::from("/a.flac"),
        },
        SourceIssue::MissingTags {
            path: PathBuf::from("/a.flac"),
            tags: vec!["composer".to_owned()],
        },
        SourceIssue::MissingTags {
            path: PathBuf::from("/a.flac"),
            tags: vec!["composer".to_owned(), "disc_number".to_owned()],
        },
        SourceIssue::FlacError {
            path: PathBuf::from("/a.flac"),
            error: "e".to_owned(),
        },
        SourceIssue::SampleRate {
            path: PathBuf::from("/a.flac"),
            rate: 192_000,
        },
        SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("CD1"),
        },
        SourceIssue::Scene,
    ]
}
