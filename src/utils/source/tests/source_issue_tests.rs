use crate::utils::SourceIssue::{ApiResponse, Id, IdError, Provider};
use crate::utils::{IdProviderError, SourceIssue};
use gazelle_api::GazelleError::*;

#[test]
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
        Provider(BadRequest),
        Provider(NotFound),
        Provider(Deserialization("A deserialization error occured".to_owned())),
        Provider(Request("A request error occured".to_owned())),
        Provider(Unexpected(503, "Service unavailable".to_owned())),
        Provider(Empty(500)),
    ];
    let expected = "- type: id_error
  details: Hello, world!
- type: id
  NoMatch: null
- type: id
  UrlInvalid: null
- type: id
  TorrentFileSource:
    actual: ABC
    expected: CBA
- type: api_response
  action: test
  status_code: 200
  error: test
- type: provider
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

    // Act
    let yaml = serde_yaml::to_string(&example)?;
    println!("{yaml}");
    let deserialized: Vec<SourceIssue> = serde_yaml::from_str(expected)?;

    // Assert
    assert_eq!(yaml, expected);
    assert_eq!(deserialized.len(), example.len());
    Ok(())
}
