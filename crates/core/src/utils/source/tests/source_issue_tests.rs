#[allow(deprecated)]
use crate::testing_prelude::*;
#[allow(deprecated)]
use crate::utils::SourceIssue::{Api, ApiResponse, Id, IdError, Provider};
use gazelle_api::GazelleSerializableError::*;

#[test]
#[allow(deprecated)]
fn source_issue_serialization() -> Result<(), serde_yaml::Error> {
    // Arrange
    let example = vec![
        IdError {
            details: "Hello, world!".to_owned(),
        },
        Id(IdProviderError::NoMatch),
        Id(IdProviderError::UrlInvalid),
        Id(IdProviderError::TorrentFileSource {
            actual: "ABC".to_owned(),
            expected: "CBA".to_owned(),
        }),
        ApiResponse {
            action: "test".to_owned(),
            status_code: 200,
            error: "test".to_owned(),
        },
        Provider,
        Api {
            response: BadRequest {
                message: String::new(),
            },
        },
        Api {
            response: NotFound {
                message: String::new(),
            },
        },
        Api {
            response: Deserialization {
                error: "A deserialization error occured".to_owned(),
            },
        },
        Api {
            response: Request {
                error: "A request error occured".to_owned(),
            },
        },
        Api {
            response: Other {
                status: 503,
                message: Some("Service unavailable".to_owned()),
            },
        },
        Api {
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
    error: A deserialization error occured
- type: api
  response:
    type: request
    error: A request error occured
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
    let yaml = serde_yaml::to_string(&example)?;
    println!("{yaml}");
    let deserialized: Vec<SourceIssue> = serde_yaml::from_str(expected)?;

    // Assert
    assert_eq!(yaml, expected);
    assert_eq!(deserialized.len(), example.len());
    Ok(())
}

#[test]
#[allow(deprecated, clippy::similar_names)]
fn source_issue_provider_deprecated() -> Result<(), serde_yaml::Error> {
    // Arrange
    let example = vec![
        Api {
            response: BadRequest {
                message: String::new(),
            },
        },
        Api {
            response: NotFound {
                message: String::new(),
            },
        },
        Api {
            response: Deserialization {
                error: "A deserialization error occured".to_owned(),
            },
        },
        Api {
            response: Request {
                error: "A request error occured".to_owned(),
            },
        },
        Api {
            response: Other {
                status: 503,
                message: Some("Service unavailable".to_owned()),
            },
        },
        Api {
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
  Deserialization: A deserialization error occured
- type: provider
  Request: A request error occured
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
    error: A deserialization error occured
- type: api
  response:
    type: request
    error: A request error occured
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
    let before_deserialized: Vec<SourceIssue> = serde_yaml::from_str(before)?;
    let after_deserialized: Vec<SourceIssue> = serde_yaml::from_str(after)?;
    let yaml = serde_yaml::to_string(&example)?;
    println!("{yaml}");
    let before_reserialized = serde_yaml::to_string(&before_deserialized)?;
    println!("--------------------");
    println!("{before_reserialized}");

    // Assert
    assert_eq!(yaml, after);
    assert_eq!(before_deserialized.len(), after_deserialized.len());
    Ok(())
}
