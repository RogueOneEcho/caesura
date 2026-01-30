use crate::testing_prelude::*;
use crate::utils::SourceIssue::*;

#[test]
fn test_serialize_source_rules_vec() {
    // Arrange
    let mut existing_formats = BTreeSet::new();
    existing_formats.insert(ExistingFormat::_320);
    existing_formats.insert(ExistingFormat::Flac);
    let file = PathBuf::from("/path/to/file.flac");
    let rules: Vec<SourceIssue> = vec![
        Category {
            actual: "Music".to_owned(),
        },
        Scene,
        LossyMaster,
        LossyWeb,
        Trumpable,
        Existing {
            formats: existing_formats,
        },
        MissingDirectory {
            path: PathBuf::from("/path/to/source"),
        },
        NoFlacs {
            path: PathBuf::from("/path/to/source"),
        },
        Imdl {
            details: "abcd1234".to_owned(),
        },
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
        Channels {
            path: file.clone(),
            count: 2,
        },
    ];

    // Act
    let yaml = serde_yaml::to_string(&rules).expect("Failed to serialize SourceIssue");
    println!("{yaml}");

    // Assert
    let expected = r"- type: category
  actual: Music
- type: scene
- type: lossy_master
- type: lossy_web
- type: trumpable
- type: existing
  formats:
  - flac
  - '320'
- type: missing_directory
  path: /path/to/source
- type: no_flacs
  path: /path/to/source
- type: imdl
  details: abcd1234
- type: length
  path: /path/to/file
  excess: 10
- type: missing_tags
  path: /path/to/file.flac
  tags:
  - Title
  - Artist
- type: flac_error
  path: /path/to/file.flac
  error: I/O Error
- type: sample_rate
  path: /path/to/file.flac
  rate: 44100
- type: channels
  path: /path/to/file.flac
  count: 2
";
    assert_eq!(yaml, expected);
}
