use crate::formats::TargetFormat::{Flac, V0, _320};
use crate::formats::TargetFormatProvider;
use crate::fs::DirectoryReader;
use crate::hosting::HostBuilder;
use crate::options::{CopyOptions, SharedOptions, SourceArg, TargetOptions};
use crate::source::SourceProvider;
use crate::testing::options::TestOptionsFactory;
use crate::testing::*;
use crate::transcode::TranscodeCommand;
use rogue_logging::Error;
use std::fs::metadata;
use std::os::unix::prelude::MetadataExt;

#[tokio::test]
async fn transcode_command() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let source_options = TestOptionsFactory::from(SourceArg {
        source: Some("206675".to_owned()),
    });
    let shared_options = TestOptionsFactory::from(SharedOptions {
        output: Some(TempDirectory::create("caesura")),
        ..SharedOptions::default()
    });
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        target: Some(vec![Flac, _320, V0]),
    });
    let copy_options = TestOptionsFactory::from(CopyOptions {
        hard_link: Some(true),
    });
    let output_dir = shared_options.output.clone().expect("output should be set");
    let host = HostBuilder::new()
        .with_options(source_options)
        .with_options(shared_options.clone())
        .with_options(target_options)
        .with_options(copy_options)
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
        .with_extension("jpg")
        .read(&output_dir)
        .expect("Should be able to read dir");
    assert_eq!(generated_files.len(), target_count);
    let cover = generated_files.first().expect("should be at least one");
    let hard_links = metadata(cover)
        .expect("should be able to get metadata")
        .nlink();
    assert_eq!(hard_links, 2);
    Ok(())
}
