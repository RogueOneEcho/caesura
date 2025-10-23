use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use log::trace;

use crate::utils::TargetFormat::*;
use rogue_logging::Error;

#[tokio::test]
async fn transcode_command() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let source_options = TestOptionsFactory::from(SourceArg {
        source: Some("206675".to_owned()),
    });
    let shared_options = TestOptionsFactory::from(SharedOptions {
        output: Some(TempDirectory::create("transcode_command")),
        ..SharedOptions::default()
    });
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        target: Some(vec![Flac, _320, V0]),
    });
    let output_dir = shared_options.output.clone().expect("output should be set");
    let host = HostBuilder::new()
        .with_options(source_options)
        .with_options(shared_options.clone())
        .with_options(target_options)
        .build();
    let provider = host.services.get_required_mut::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_from_options()
        .await
        .expect("Source provider should not fail");

    // Act
    transcoder.execute_cli().await?;

    // Assert
    let generated_files = DirectoryReader::new()
        .with_extensions(vec!["flac", "mp3"])
        .read(&output_dir)
        .expect("Should be able to read dir");
    let targets = host.services.get_required::<TargetFormatProvider>();
    let target_count = targets.get(source.format, &source.existing).len();
    let expected_file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len()
        * target_count;
    assert_eq!(generated_files.len(), expected_file_count);
    let generated_files = DirectoryReader::new()
        .with_extensions(IMAGE_EXTENSIONS.to_vec())
        .read(&output_dir)
        .expect("Should be able to read dir");
    trace!(
        "{}",
        generated_files
            .iter()
            .map(|f| f.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(generated_files.len(), target_count * 2);
    Ok(())
}
