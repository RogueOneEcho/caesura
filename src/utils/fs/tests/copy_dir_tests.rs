use crate::utils::*;

use rogue_logging::Error;
use std::fs::read_dir;
use std::path::PathBuf;

#[tokio::test]
async fn test_copy_dir() -> Result<(), Error> {
    // Arrange
    let source_dir = read_dir(PathBuf::from("./content"))
        .map_err(|e| io_error(e, "read source dir"))?
        .filter_map(Result::ok) // Filter out errors
        .find(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .expect("Should have a directory");
    let target_dir = TempDirectory::create("caesura").join("target");

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
