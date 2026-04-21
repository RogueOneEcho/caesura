#[allow(deprecated)]
use crate::testing_prelude::*;
use gazelle_api::GazelleSerializableError::*;

#[test]
#[allow(deprecated)]
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
