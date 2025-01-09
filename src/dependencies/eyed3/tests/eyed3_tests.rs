use super::super::*;

use rogue_logging::Error;
use std::path::PathBuf;

use crate::utils::*;
#[tokio::test]
#[ignore]
async fn show() -> Result<(), Error> {
    // Arrange
    let paths = DirectoryReader::new()
        .with_extension("mp3")
        .read(&PathBuf::from("./output"))
        .expect("Directory should exist");
    let path = paths.first().expect("Should be at least one sample");

    // Act
    println!("{path:?}");
    let output = EyeD3Command::display(path).await?;
    println!("{output}");

    // Assert
    assert!(!output.is_empty());

    Ok(())
}
