use di::ServiceProvider;

use crate::errors::AppError;
use crate::logging::*;
use crate::options::SubCommand::*;
use crate::options::{Arguments, Options, SharedOptions, SpectrogramOptions, TranscodeOptions};
use crate::source;
use crate::source::Source;
use crate::spectrogram::SpectrogramGenerator;
use crate::transcode::SourceTranscoder;
use crate::verify::SourceVerifier;

/// Application host, responsible for executing the application
///
/// [`HostBuilder`] takes care of building the [Host] and loading the
/// dependency injection [`ServiceProvider`].
pub struct Host {
    /// Dependency injection service provider
    pub services: ServiceProvider,
}

impl Host {
    #[must_use]
    pub fn new(services: ServiceProvider) -> Self {
        Host { services }
    }

    /// Execute the application
    ///
    /// 1. Configure logging
    /// 2. Determine the command to execute
    /// 3. Execute the command
    pub async fn execute(&self) -> Result<bool, AppError> {
        let logger = self.services.get_required::<Logger>();
        Logger::init(logger);
        let options = self.services.get_required::<SharedOptions>();
        if !options.validate() {
            return Ok(false);
        }
        let source_provider = self.services.get_required_mut::<source::SourceProvider>();
        let source_input = options.source.clone().unwrap_or_default();
        let source = source_provider
            .write()
            .expect("Source provider should be writeable")
            .get_by_string(&source_input)
            .await?;
        match Arguments::get_command_or_exit() {
            Spectrogram { .. } => self.execute_spectrogram(&source).await,
            Transcode { .. } => self.execute_transcode(&source).await,
            Verify { .. } => self.execute_verify(&source).await,
        }
    }

    async fn execute_spectrogram(&self, source: &Source) -> Result<bool, AppError> {
        let options = self.services.get_required::<SpectrogramOptions>();
        if !options.validate() {
            return Ok(false);
        }
        let service = self.services.get_required::<SpectrogramGenerator>();
        service.execute(source).await
    }

    async fn execute_transcode(&self, source: &Source) -> Result<bool, AppError> {
        let options = self.services.get_required::<TranscodeOptions>();
        if !options.validate() {
            return Ok(false);
        }
        let service = self.services.get_required::<SourceTranscoder>();
        service.execute(source).await
    }

    async fn execute_verify(&self, source: &Source) -> Result<bool, AppError> {
        let options = self.services.get_required::<TranscodeOptions>();
        if !options.validate() {
            return Ok(false);
        }
        let service = self.services.get_required_mut::<SourceVerifier>();
        let mut service = service
            .write()
            .expect("SourceVerifier should be available to write");
        service.execute(source).await
    }
}
