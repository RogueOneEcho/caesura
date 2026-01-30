use crate::commands::*;
use crate::hosting::*;
use crate::utils::*;

/// Test that `ConfigCommand` serializes all option types to YAML.
#[test]
fn config_command_serializes_default_options() {
    // Arrange
    let _ = init_logger();
    let host = HostBuilder::new().build();
    let config_command = host.services.get_required::<ConfigCommand>();

    // Act
    let result = config_command.execute();

    // Assert
    assert!(result.is_ok());
    assert!(result.expect("should return bool"));
}
