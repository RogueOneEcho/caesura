use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;
use rogue_logging::{TimeFormat, Verbosity};
use std::path::PathBuf;

/// Test that `ConfigCommand` serializes all option types to YAML.
#[tokio::test]
async fn config_command_serializes_default_options() {
    // Arrange
    let _ = init_logger();
    let host = HostBuilder::new()
        .with_options(SharedOptions {
            verbosity: Some(Verbosity::Info),
            log_time: Some(TimeFormat::None),
            indexer: Some("red".to_owned()),
            indexer_url: Some("https://example.com".to_owned()),
            api_key: Some("test_key".to_owned()),
            content: Some(vec![PathBuf::from("/content")]),
            output: Some(PathBuf::from("/output")),
            ..SharedOptions::default()
        })
        .with_options(BatchOptions::default())
        .with_options(CacheOptions::default())
        .with_options(FileOptions::default())
        .with_options(RunnerOptions { cpus: Some(4) })
        .with_options(SpectrogramOptions::default())
        .with_options(TargetOptions::default())
        .with_options(UploadOptions::default())
        .with_options(VerifyOptions::default())
        .build();
    let config_command = host.services.get_required::<ConfigCommand>();

    // Act
    let result = config_command.execute();

    // Assert
    assert!(result.is_ok());
    assert!(result.expect("should return bool"));
}
