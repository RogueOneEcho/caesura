use crate::utils::SourceIssue::Id;
use crate::utils::{IdProviderError, SourceIssue};

#[test]
#[allow(clippy::similar_names)]
fn id_provider_error_pascal_case_alias() -> Result<(), serde_yaml::Error> {
    // Arrange
    let example = vec![
        Id(IdProviderError::NoMatch),
        Id(IdProviderError::UrlInvalid),
        Id(IdProviderError::TorrentFileSource {
            actual: "ABC".to_owned(),
            expected: "CBA".to_owned(),
        }),
    ];
    let pascal_case = "- type: id
  NoMatch: null
- type: id
  UrlInvalid: null
- type: id
  TorrentFileSource:
    actual: ABC
    expected: CBA
";
    let snake_case = "- type: id
  no_match: null
- type: id
  url_invalid: null
- type: id
  torrent_file_source:
    actual: ABC
    expected: CBA
";

    // Act
    let pascal_case_deserialized: Vec<SourceIssue> = serde_yaml::from_str(pascal_case)?;
    let snake_case_deserialized: Vec<SourceIssue> = serde_yaml::from_str(snake_case)?;
    let yaml = serde_yaml::to_string(&example)?;
    println!("{yaml}");
    let pascal_case_reserialized = serde_yaml::to_string(&pascal_case_deserialized)?;

    // Assert
    assert_eq!(yaml, snake_case);
    assert_eq!(
        pascal_case_deserialized.len(),
        snake_case_deserialized.len()
    );
    assert_eq!(pascal_case_reserialized, snake_case);
    Ok(())
}
