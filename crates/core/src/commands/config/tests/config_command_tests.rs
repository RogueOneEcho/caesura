use std::path::PathBuf;

use colored::control;

use crate::testing_prelude::*;

/// Test that `ConfigCommand` renders documented YAML with comments.
#[test]
fn config_command_renders_documented_yaml() {
    // Arrange
    init_logger();
    control::set_override(false);
    let host = HostBuilder::new()
        .with_options(SharedOptions {
            output: PathBuf::from("/test/output"),
            ..SharedOptions::mock()
        })
        .with_options(CacheOptions {
            cache: PathBuf::from("/test/cache"),
        })
        .with_options(RunnerOptions { cpus: Some(4) })
        .with_options(SoxOptions {
            sox_path: None,
            sox_ng: true,
        })
        .expect_build();
    let config_command = host.services.get_required::<ConfigCommand>();

    // Act
    let output = config_command.render().expect("render should succeed");

    // Assert
    insta::assert_snapshot!(output);
    control::unset_override();
}
