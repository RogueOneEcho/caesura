use rogue_logging::Error;
use std::path::PathBuf;

use crate::dependencies::*;
use crate::utils::*;
#[tokio::test]
#[ignore]
async fn show() -> Result<(), Error> {
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
