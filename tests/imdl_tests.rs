use red_oxide::imdl::imdl_command::ImdlCommand;
use red_oxide::imdl::ImdlError;
use red_oxide::testing::get_sample_torrent_file;

#[tokio::test]
async fn show() -> Result<(), ImdlError> {
    // Arrange
    let path = get_sample_torrent_file().expect("Path should be valid");

    // Act
    let summary = ImdlCommand::show(&path).await?;

    // Assert
    assert!(!summary.files.is_empty());

    Ok(())
}
