use std::path::PathBuf;

use red_oxide::fs::DirectoryReader;
use red_oxide::imdl::imdl_command::ImdlCommand;
use red_oxide::imdl::ImdlError;
use red_oxide::testing::TORRENTS_SAMPLES_DIR;

#[tokio::test]
async fn show() -> Result<(), ImdlError> {
    // Arrange
    let paths = DirectoryReader::new()
        .with_extension("torrent")
        .read(&PathBuf::from(TORRENTS_SAMPLES_DIR))
        .expect("Directory should exist");
    let path = paths.first().expect("Should be at least one sample");

    // Act
    let summary = ImdlCommand::show(path).await?;

    // Assert
    assert!(!summary.files.is_empty());

    Ok(())
}
