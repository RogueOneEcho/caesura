use crate::utils::*;

use rogue_logging::Error;
use std::path::PathBuf;

#[tokio::test]
async fn test_copy_dir() -> Result<(), Error> {
    // Arrange
    let source_dir = PathBuf::from("./samples/content");
    let target_dir = TempDirectory::create("test_copy_dir").join("target");
    assert!(
        source_dir.is_dir(),
        "Sample directory should exist: {}",
        source_dir.display()
    );

    // Act
    copy_dir(&source_dir, &target_dir, false).await?;

    // Assert
    let source_files: Vec<PathBuf> = DirectoryReader::new()
        .read(&source_dir)
        .expect("Should be able to read dir");
    let target_files = DirectoryReader::new()
        .read(&target_dir)
        .expect("Should be able to read source dir");
    assert_eq!(source_files.len(), target_files.len());
    Ok(())
}
