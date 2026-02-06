use crate::commands::CommandArguments::Queue;
use crate::commands::CommandArguments::*;
use crate::commands::QueueCommandArguments::*;
use crate::prelude::*;
use di::ServiceProvider;
use miette::Report;
use rogue_logging::Logger;

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
    pub async fn execute(&self) -> Result<bool, Report> {
        let _ = self.services.get_required::<Logger>();
        match ArgumentsParser::get_or_show_help() {
            Config => self
                .services
                .get_required::<ConfigCommand>()
                .execute()
                .map_err(Report::new),
            Docs => Ok(self.services.get_required::<DocsCommand>().execute()),
            Batch { .. } => self
                .services
                .get_required::<BatchCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Queue {
                command: Add { .. },
            } => self
                .services
                .get_required::<QueueAddCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Queue {
                command: List { .. },
            } => self
                .services
                .get_required::<QueueListCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Queue {
                command: Remove { .. },
            } => self
                .services
                .get_required::<QueueRemoveCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Queue {
                command: Summary { .. },
            } => self
                .services
                .get_required::<QueueSummaryCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Spectrogram { .. } => self
                .services
                .get_required::<SpectrogramCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Transcode { .. } => self
                .services
                .get_required::<TranscodeCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Upload { .. } => self
                .services
                .get_required::<UploadCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
            Verify { .. } => self
                .services
                .get_required::<VerifyCommand>()
                .execute_cli()
                .await
                .map_err(Report::new),
        }
    }
}
