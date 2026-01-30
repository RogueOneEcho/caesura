use crate::testing_prelude::*;

/// Test that `JobRunner` is created with correct concurrency.
#[test]
fn job_runner_created_with_correct_concurrency() {
    // Arrange
    let host = HostBuilder::new()
        .with_options(RunnerOptions { cpus: Some(4) })
        .build();

    // Act - Get the JobRunner which has the semaphore
    let runner = host.services.get_required::<JobRunner>();

    // Assert - Check semaphore permits through the runner
    assert_eq!(runner.semaphore.available_permits(), 4);
}

/// Test that `JobRunner` execute returns Ok with empty job set.
#[tokio::test]
async fn job_runner_execute_empty_succeeds() -> Result<(), Error> {
    // Arrange
    let host = HostBuilder::new().build();
    let runner = host.services.get_required::<JobRunner>();

    // Act
    let result = runner.execute().await;

    // Assert
    assert!(result.is_ok());
    Ok(())
}

/// Test that `JobRunner` `execute_without_publish` returns Ok with empty job set.
#[tokio::test]
async fn job_runner_execute_without_publish_empty_succeeds() -> Result<(), Error> {
    // Arrange
    let host = HostBuilder::new().build();
    let runner = host.services.get_required::<JobRunner>();

    // Act
    let result = runner.execute_without_publish().await;

    // Assert
    assert!(result.is_ok());
    Ok(())
}
