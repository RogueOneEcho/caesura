use crate::testing_prelude::*;

#[test]
#[allow(clippy::similar_names)]
fn id_provider_error_pascal_case_alias() -> Result<(), YamlError> {
    // Arrange
    let example = vec![
        SourceIssue::Id(IdProviderError::NoMatch),
        SourceIssue::Id(IdProviderError::UrlInvalid),
        SourceIssue::Id(IdProviderError::TorrentFileSource {
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
    let pascal_case_deserialized: Vec<SourceIssue> = yaml_from_str(pascal_case)?;
    let snake_case_deserialized: Vec<SourceIssue> = yaml_from_str(snake_case)?;
    let yaml = yaml_to_string(&example)?;
    println!("{yaml}");
    let pascal_case_reserialized = yaml_to_string(&pascal_case_deserialized)?;

    // Assert
    assert_eq!(yaml, snake_case);
    assert_eq!(
        pascal_case_deserialized.len(),
        snake_case_deserialized.len()
    );
    assert_eq!(pascal_case_reserialized, snake_case);
    Ok(())
}
