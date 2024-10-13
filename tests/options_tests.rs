use caesura::logging::{Logger, Trace, Verbosity};
use caesura::options::*;

const LOG_LEVEL: Verbosity = Trace;

#[tokio::test]
async fn batch_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let batch_options = provider.get::<BatchOptions>();

    // Assert
    assert!(batch_options.validate());
}

#[tokio::test]
async fn file_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let file_options = provider.get::<FileOptions>();

    // Assert
    assert!(file_options.validate());
}

#[tokio::test]
async fn runner_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let runner_options = provider.get::<RunnerOptions>();

    // Assert
    assert!(runner_options.validate());
}

#[tokio::test]
async fn spectrogram_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let spectrogram_options = provider.get::<SpectrogramOptions>();

    // Assert
    assert!(spectrogram_options.validate());
}

#[tokio::test]
async fn target_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let target_options = provider.get::<TargetOptions>();

    // Assert
    assert!(target_options.validate());
}

#[tokio::test]
async fn upload_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let upload_options = provider.get::<UploadOptions>();

    // Assert
    assert!(upload_options.validate());
}

#[tokio::test]
async fn verify_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let verify_options = provider.get::<VerifyOptions>();

    // Assert
    assert!(verify_options.validate());
}

#[tokio::test]
async fn shared_options_validate() {
    // Arrange
    Logger::init_new(LOG_LEVEL);
    let provider = OptionsProvider::new();

    // Act
    let shared_options = provider.get::<SharedOptions>();

    // Assert
    assert!(shared_options.validate());
}
