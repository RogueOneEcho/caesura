use crate::built_info::PKG_NAME;
use crate::fs::DirectoryReader;
use crate::hosting::HostBuilder;
use crate::options::TargetOptions;
use crate::source::*;
use crate::testing::options::TestOptionsFactory;
use rogue_logging::Error;
use rogue_logging::Logger;

#[tokio::test]
async fn source_provider() -> Result<(), Error> {
    // Arrange
    Logger::force_init(PKG_NAME.to_owned());
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        ..TargetOptions::default()
    });
    let host = HostBuilder::new().with_options(target_options).build();
    let provider = host.services.get_required_mut::<SourceProvider>();

    // Act
    let source = provider
        .write()
        .expect("Source provider should be writeable")
        .get_from_options()
        .await
        .unwrap();

    // Assert
    let file_count = DirectoryReader::new()
        .with_extension("flac")
        .read(&source.directory)
        .expect("Should be able to read source dir")
        .len();
    assert!(file_count > 0);
    Ok(())
}
