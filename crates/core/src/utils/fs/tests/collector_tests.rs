use crate::testing_prelude::*;

#[test]
fn collector_collect_flacs_missing_directory() {
    // Arrange
    let mut source = Source::mock();
    source.directory = PathBuf::from("/nonexistent/path");

    // Act + Assert
    assert_eq!(
        Collector::collect_flacs(&source).err(),
        Some(SourceIssue::MissingDirectory {
            path: PathBuf::from("/nonexistent/path")
        })
    );
}

#[test]
fn collector_collect_flacs_empty_directory() {
    // Arrange
    let empty_dir = TempDirectory::create("collector_collect_flacs_empty_directory");
    let mut source = Source::mock();
    source.directory = empty_dir.to_path_buf();

    // Act + Assert
    assert_eq!(
        Collector::collect_flacs(&source).err(),
        Some(SourceIssue::NoFlacs {
            path: empty_dir.to_path_buf()
        })
    );
}
