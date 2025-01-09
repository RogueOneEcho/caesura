use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{debug, info};

use rogue_logging::Error;

use crate::commands::*;
use crate::options::*;
use crate::utils::*;

/// Generate spectrograms for each track of a FLAC source.
#[injectable]
pub(crate) struct SpectrogramCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    source_provider: RefMut<SourceProvider>,
    paths: Ref<PathManager>,
    factory: Ref<SpectrogramJobFactory>,
    runner: Ref<JobRunner>,
}

impl SpectrogramCommand {
    /// Execute [`SpectrogramCommand`] from the CLI.
    ///
    /// [`Source`] is retrieved from the CLI arguments.
    ///
    /// Returns `true` if the spectrogram generation succeeds.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Error> {
        if !self.arg.validate()
            || !self.shared_options.validate()
            || !self.spectrogram_options.validate()
        {
            return Ok(false);
        }
        let source = self
            .source_provider
            .write()
            .expect("Source provider should be writeable")
            .get_from_options()
            .await
            .map_err(|e| error("get source from options", e.to_string()))?;
        let status = self.execute(&source).await;
        if let Some(error) = &status.error {
            error.log();
        }
        Ok(status.success)
    }

    /// Execute [`SpectrogramCommand`] on a [`Source`].
    ///
    /// Returns a [`SpectrogramStatus`] indicating the success of the operation and any errors.
    ///
    /// Errors are not logged so should be handled by the caller.
    #[must_use]
    pub(crate) async fn execute(&self, source: &Source) -> SpectrogramStatus {
        info!("{} spectrograms for {}", "Creating".bold(), source);
        let collection = Collector::get_flacs(&source.directory);
        let jobs = self.factory.create(&collection, source);
        let count = jobs.len();
        self.runner.add(jobs);
        match self.runner.execute().await {
            Ok(()) => {
                info!("{} {count} spectrograms for {source}", "Created".bold());
                let path = self.paths.get_spectrogram_dir(source);
                let path_display = path.to_string_lossy().to_string();
                debug!("in {path_display}");
                SpectrogramStatus {
                    success: true,
                    path: Some(path),
                    count,
                    completed: TimeStamp::now(),
                    error: None,
                }
            }
            Err(error) => SpectrogramStatus {
                success: false,
                path: None,
                count,
                completed: TimeStamp::now(),
                error: Some(error),
            },
        }
    }
}
