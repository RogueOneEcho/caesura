use crate::prelude::*;
use crate::utils::Job::Additional;
use rogue_logging::Colors;
use tokio::fs::{copy, hard_link};

/// Transcode each track of a FLAC source to the target formats.
#[injectable]
pub(crate) struct TranscodeCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    source_provider: Ref<SourceProvider>,
    copy_options: Ref<CopyOptions>,
    file_options: Ref<FileOptions>,
    paths: Ref<PathManager>,
    targets: Ref<TargetFormatProvider>,
    transcode_job_factory: Ref<TranscodeJobFactory>,
    additional_job_factory: Ref<AdditionalJobFactory>,
    runner: Ref<JobRunner>,
}

impl TranscodeCommand {
    /// Execute [`TranscodeCommand`] from the CLI.
    ///
    /// [`Source`] is retrieved from the CLI arguments.
    ///
    /// Returns `true` if all the transcodes succeed.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<TranscodeAction>> {
        if !self.arg.validate() {
            return Ok(false);
        }
        let source = self
            .source_provider
            .get_from_options()
            .await
            .map_err(Failure::wrap(TranscodeAction::GetSource))?
            .map_err(Failure::wrap(TranscodeAction::GetSource))?;
        self.execute(&source).await?;
        Ok(true)
    }

    /// Execute [`TranscodeCommand`] on a [`Source`].
    ///
    /// Returns a [`TranscodeSuccess`] on success, or a [`Failure`] on error.
    pub(crate) async fn execute(
        &self,
        source: &Source,
    ) -> Result<TranscodeSuccess, Failure<TranscodeAction>> {
        let targets = self.targets.get(source.format, &source.existing);
        if targets.is_empty() {
            return Err(Failure::new(
                TranscodeAction::Transcode,
                TranscodeError::NoTranscodes,
            ));
        }
        let formats: Vec<TranscodeFormatStatus> = targets
            .iter()
            .map(|&format| TranscodeFormatStatus {
                format,
                path: self.paths.get_transcode_target_dir(source, format),
            })
            .collect();
        let targets = self.skip_completed(source, &targets).await;
        if targets.is_empty() {
            return Ok(TranscodeSuccess { formats });
        }
        self.execute_transcode(source, &targets).await?;
        self.execute_additional(source, &targets).await?;
        self.execute_torrent(source, &targets)
            .await
            .map_err(Failure::wrap(TranscodeAction::CreateTorrent))?;
        Ok(TranscodeSuccess { formats })
    }

    #[must_use]
    async fn skip_completed(
        &self,
        source: &Source,
        targets: &BTreeSet<TargetFormat>,
    ) -> BTreeSet<TargetFormat> {
        let mut out: BTreeSet<TargetFormat> = BTreeSet::new();
        for target in targets {
            if let Ok(Some(path)) = self
                .paths
                .get_or_duplicate_existing_torrent_path(source, *target)
                .await
            {
                debug!("{} existing {target} transcode", "Found".bold());
                trace!("{}", path.display());
            } else {
                // Errors are intentionally ignored
                out.insert(*target);
            }
        }
        out
    }

    async fn execute_transcode(
        &self,
        source: &Source,
        targets: &BTreeSet<TargetFormat>,
    ) -> Result<(), Failure<TranscodeAction>> {
        let rename_tracks = self.file_options.rename_tracks;
        let flacs = if rename_tracks {
            Collector::get_flacs_with_context(&source.directory)
        } else {
            Collector::get_flacs(&source.directory)
        };

        info!(
            "{} to {} for {} FLACs in {}",
            "Transcoding".bold(),
            join_humanized(targets),
            flacs.len().to_string().gray(),
            source
        );
        for target in targets {
            let jobs = self.transcode_job_factory.create(&flacs, source, *target)?;
            self.runner.add(jobs);
        }
        self.runner
            .execute()
            .await
            .map_err(Failure::wrap(TranscodeAction::ExecuteRunner))?;
        info!("{} {}", "Transcoded".bold(), source);
        Ok(())
    }

    async fn execute_additional(
        &self,
        source: &Source,
        targets: &BTreeSet<TargetFormat>,
    ) -> Result<(), Failure<TranscodeAction>> {
        let files = Collector::get_additional(&source.directory);
        debug!(
            "{} {} additional files",
            "Adding".bold(),
            files.len().to_string().gray()
        );
        let first_target = targets.first().expect("should be at least one target");
        let jobs = self
            .additional_job_factory
            .create(&files, source, *first_target)
            .await?;
        let from_prefix = self.paths.get_transcode_target_dir(source, *first_target);
        self.runner.add_without_publish(jobs);
        self.runner
            .execute_without_publish()
            .await
            .map_err(Failure::wrap(TranscodeAction::ExecuteRunner))?;
        for target in targets.iter().skip(1) {
            let jobs = self
                .additional_job_factory
                .create(&files, source, *target)
                .await?;
            let output = self.paths.get_transcode_target_dir(source, *target);
            for job in jobs {
                if let Additional(AdditionalJob { resize, .. }) = job {
                    let from = from_prefix.clone().join(
                        resize
                            .output
                            .strip_prefix(&output)
                            .expect("should have prefix"),
                    );
                    let verb = if self.copy_options.hard_link {
                        hard_link(&from, &resize.output)
                            .await
                            .map_err(Failure::wrap_with_path(
                                TranscodeAction::HardLinkAdditional,
                                &resize.output,
                            ))?;
                        "Hard Linked"
                    } else {
                        copy(&from, &resize.output)
                            .await
                            .map_err(Failure::wrap_with_path(
                                TranscodeAction::CopyAdditional,
                                &resize.output,
                            ))?;
                        "Copied"
                    };
                    trace!(
                        "{} {} to {}",
                        verb.bold(),
                        from.display(),
                        resize.output.display()
                    );
                }
            }
        }
        debug!("{} additional files {}", "Added".bold(), source);

        Ok(())
    }

    async fn execute_torrent(
        &self,
        source: &Source,
        targets: &BTreeSet<TargetFormat>,
    ) -> Result<(), Failure<ImdlAction>> {
        debug!("{} torrents {}", "Creating".bold(), source);
        for target in targets {
            let content_dir = self.paths.get_transcode_target_dir(source, *target);
            let torrent_path = self.paths.get_torrent_path(source, *target);
            let announce_url = self.shared_options.announce_url.clone();
            let indexer = self.shared_options.indexer.clone();
            ImdlCommand::create(&content_dir, &torrent_path, announce_url, indexer).await?;
            trace!("{} torrent {}", "Created".bold(), torrent_path.display());
        }
        debug!("{} torrents {}", "Created".bold(), source);
        Ok(())
    }
}
