use crate::prelude::*;

/// Generate spectrograms for each track of a FLAC source.
#[injectable]
pub(crate) struct SpectrogramCommand {
    arg: Ref<SourceArg>,
    source_provider: Ref<SourceProvider>,
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
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<SpectrogramAction>> {
        if !self.arg.validate() {
            return Ok(false);
        }
        let source = self
            .source_provider
            .get_from_options()
            .await
            .map_err(Failure::wrap(SpectrogramAction::GetSource))?
            .map_err(Failure::wrap(SpectrogramAction::GetSource))?;
        self.execute(&source).await?;
        Ok(true)
    }

    /// Execute [`SpectrogramCommand`] on a [`Source`].
    ///
    /// Returns a [`SpectrogramSuccess`] on success, or a [`Failure`] on error.
    pub(crate) async fn execute(
        &self,
        source: &Source,
    ) -> Result<SpectrogramSuccess, Failure<SpectrogramAction>> {
        let path = self.paths.get_spectrogram_dir(source);
        if path.is_dir() {
            info!("{} existing spectrograms {source}", "Found".bold());
            debug!("in {}", path.display());
            return Ok(SpectrogramSuccess { path, count: 0 });
        }
        info!("{} spectrograms for {}", "Creating".bold(), source);
        let collection = Collector::get_flacs(&source.directory);
        let jobs = self.factory.create(&collection, source);
        let count = jobs.len();
        self.runner.add(jobs);
        self.runner
            .execute()
            .await
            .map_err(Failure::wrap(SpectrogramAction::ExecuteRunner))?;
        info!("{} {count} spectrograms for {source}", "Created".bold());
        debug!("in {}", path.display());
        Ok(SpectrogramSuccess { path, count })
    }
}
