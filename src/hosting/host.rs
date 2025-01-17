use di::ServiceProvider;

use rogue_logging::Error;
use rogue_logging::*;

use crate::commands::CommandArguments::Queue;
use crate::commands::CommandArguments::*;
use crate::commands::QueueCommandArguments::*;
use crate::commands::*;

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
    pub async fn execute(&self) -> Result<bool, Error> {
        let _ = self.services.get_required::<Logger>();
        match ArgumentsParser::get_or_show_help() {
            Config => self.services.get_required::<ConfigCommand>().execute(),
            Batch { .. } => {
                self.services
                    .get_required_mut::<BatchCommand>()
                    .write()
                    .expect("BatchCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Queue {
                command: Add { .. },
            } => {
                self.services
                    .get_required_mut::<QueueAddCommand>()
                    .write()
                    .expect("QueueAddCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Queue {
                command: List { .. },
            } => {
                self.services
                    .get_required_mut::<QueueListCommand>()
                    .write()
                    .expect("QueueListCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Queue {
                command: Remove { .. },
            } => {
                self.services
                    .get_required_mut::<QueueRemoveCommand>()
                    .write()
                    .expect("QueueRemoveCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Queue {
                command: Summary { .. },
            } => {
                self.services
                    .get_required_mut::<QueueSummaryCommand>()
                    .write()
                    .expect("QueueSummaryCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Spectrogram { .. } => {
                self.services
                    .get_required::<SpectrogramCommand>()
                    .execute_cli()
                    .await
            }
            Transcode { .. } => {
                self.services
                    .get_required::<TranscodeCommand>()
                    .execute_cli()
                    .await
            }
            Upload { .. } => {
                self.services
                    .get_required_mut::<UploadCommand>()
                    .write()
                    .expect("UploadCommand should be available to write")
                    .execute_cli()
                    .await
            }
            Verify { .. } => {
                self.services
                    .get_required_mut::<VerifyCommand>()
                    .write()
                    .expect("VerifyCommand should be available to write")
                    .execute_cli()
                    .await
            }
        }
    }
}
