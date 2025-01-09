use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;

use rogue_logging::Error;

#[tokio::test]
async fn verify_command() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let target_options = TestOptionsFactory::from(TargetOptions {
        allow_existing: Some(true),
        ..TargetOptions::default()
    });
    let host = HostBuilder::new().with_options(target_options).build();
    let verifier = host.services.get_required_mut::<VerifyCommand>();
    let mut verifier = verifier
        .write()
        .expect("verifier should be available to write");

    // Act
    let _is_verified = verifier.execute_cli().await?;

    // Assert not required
    Ok(())
}
