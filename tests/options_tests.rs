use caesura::errors::AppError;
use caesura::logging::{Debug, Logger};
use caesura::options::{BatchOptions, FileOptions, Options, OptionsProvider, RunnerOptions, SharedOptions, SpectrogramOptions, TargetOptions, UploadOptions, VerifyOptions};
use caesura::testing::*;

#[tokio::test]
async fn validate_options() -> Result<(), AppError> {
    // Arrange
    Logger::init_new(Debug);
    let provider = OptionsProvider::new();
    let shared_options = TestOptionsFactory::from_with_env(SharedOptions {
        verbosity: Some(Debug),
        ..SharedOptions::default()
    });
    
    // Act
    let batch_options = provider.get::<BatchOptions>();
    let file_options = provider.get::<FileOptions>();
    let runner_options = provider.get::<RunnerOptions>();
    let spectrogram_options = provider.get::<SpectrogramOptions>();
    let target_options = provider.get::<TargetOptions>();
    let upload_options = provider.get::<UploadOptions>();
    let verify_options = provider.get::<VerifyOptions>();    

    // Assert
    assert!(shared_options.validate());
    assert!(batch_options.validate());
    assert!(file_options.validate());
    assert!(runner_options.validate());
    assert!(spectrogram_options.validate());
    assert!(target_options.validate());
    assert!(upload_options.validate());
    assert!(verify_options.validate());
    
    Ok(())
}
