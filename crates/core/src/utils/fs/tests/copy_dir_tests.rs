use crate::testing_prelude::*;

#[tokio::test]
async fn test_copy_dir() -> Result<(), Error> {
    // Arrange
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let source_dir = album.source_dir();
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
