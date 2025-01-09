use rogue_logging::Error;
use std::path::PathBuf;

use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;

#[tokio::test]
async fn spectrogram_command() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let shared_options = TestOptionsFactory::from(SharedOptions {
        output: Some(TempDirectory::create("caesura")),
        ..SharedOptions::default()
    });
    let output_dir = shared_options.output.clone().expect("Should have value");
    let host = HostBuilder::new()
        .with_options(shared_options.clone())
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();
    let generator = host.services.get_required::<SpectrogramCommand>();
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_from_options()
        .await
        .expect("Source provider should not fail");

    // Act
    generator.execute_cli().await?;

    // Assert
    let generated_files: Vec<PathBuf> = DirectoryReader::new()
        .with_extension("png")
        .read(&output_dir)
        .expect("Should be able to read dir");
    let expected_file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len()
        * 2;
    assert_eq!(generated_files.len(), expected_file_count);
    Ok(())
}
