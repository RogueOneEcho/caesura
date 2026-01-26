use crate::commands::*;
use crate::dependencies::*;
use crate::options::*;
use crate::utils::*;

use crate::utils::Job::Additional;
use colored::Colorize;
use di::{Ref, injectable};
use log::*;
use rogue_logging::Colors;
use rogue_logging::Error;
use std::collections::BTreeSet;
use tokio::fs::{copy, hard_link};

/// Transcode each track of a FLAC source to the target formats.
#[injectable]
pub(crate) struct TranscodeCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    target_options: Ref<TargetOptions>,
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
    /// Returns `true` if all the transcodes succeeds.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Error> {
        if !self.arg.validate()
            || !self.shared_options.validate()
            || !self.target_options.validate()
            || !self.file_options.validate()
        {
            return Ok(false);
        }
        let source = self
            .source_provider
            .get_from_options()
            .await
            .map_err(|e| error("get source from options", e.to_string()))?;
        let status = self.execute(&source).await;
        if let Some(error) = &status.error {
            error.log();
        }
        Ok(status.success)
    }

    /// Execute [`TranscodeCommand`] on a [`Source`].
    ///
    /// Returns a [`TranscodeStatus`] indicating the success of the operation and any errors.
    ///
    /// Errors are not logged so should be handled by the caller.
    #[must_use]
    pub(crate) async fn execute(&self, source: &Source) -> TranscodeStatus {
        let targets = self.targets.get(source.format, &source.existing);
        let mut status = TranscodeStatus {
            success: false,
            formats: None,
            completed: TimeStamp::now(),
            error: None,
        };
        if targets.is_empty() {
            status.error = Some(error("transcode", "No transcodes to perform".to_owned()));
            return status;
        }
        let formats: Vec<TranscodeFormatStatus> = targets
            .iter()
            .map(|&format| TranscodeFormatStatus {
                format,
                path: self.paths.get_transcode_target_dir(source, format),
            })
            .collect();
        status.formats = Some(formats);
        let targets = self.skip_completed(source, &targets).await;
        if targets.is_empty() {
            status.success = true;
            return status;
        }
        if let Err(error) = self.execute_transcode(source, &targets).await {
            status.error = Some(error);
            status.completed = TimeStamp::now();
            return status;
        }
        if let Err(error) = self.execute_additional(source, &targets).await {
            status.error = Some(error);
            status.completed = TimeStamp::now();
            return status;
        }
        if let Err(error) = self.execute_torrent(source, &targets).await {
            status.error = Some(error);
            status.completed = TimeStamp::now();
            return status;
        }
        status.success = true;
        status
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
    ) -> Result<(), Error> {
        let flacs = Collector::get_flacs(&source.directory);
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
        self.runner.execute().await?;
        info!("{} {}", "Transcoded".bold(), source);
        Ok(())
    }

    async fn execute_additional(
        &self,
        source: &Source,
        targets: &BTreeSet<TargetFormat>,
    ) -> Result<(), Error> {
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
        self.runner.execute_without_publish().await?;
        let hard_link_option = self
            .copy_options
            .hard_link
            .expect("hard_link should be set");
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
                    let verb = if hard_link_option {
                        hard_link(&from, &resize.output)
                            .await
                            .map_err(|e| io_error(e, "hard link additional file"))?;
                        "Hard Linked"
                    } else {
                        copy(&from, &resize.output)
                            .await
                            .map_err(|e| io_error(e, "copy additional file"))?;
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
    ) -> Result<(), Error> {
        debug!("{} torrents {}", "Creating".bold(), source);
        for target in targets {
            let content_dir = self.paths.get_transcode_target_dir(source, *target);
            let path_without_indexer = self.paths.get_torrent_path(source, *target, false);
            let announce_url = self
                .shared_options
                .announce_url
                .clone()
                .expect("announce_url should be set");
            let indexer = self
                .shared_options
                .indexer
                .clone()
                .expect("indexer should be set");
            ImdlCommand::create(&content_dir, &path_without_indexer, announce_url, indexer).await?;
            trace!(
                "{} torrent {}",
                "Created".bold(),
                path_without_indexer.display()
            );
            let path_with_indexer = self.paths.get_torrent_path(source, *target, true);
            copy(&path_without_indexer, &path_with_indexer)
                .await
                .map_err(|e| io_error(e, "copy torrent file"))?;
            trace!(
                "{} torrent {}",
                "Created".bold(),
                path_with_indexer.display()
            );
        }
        debug!("{} torrents {}", "Created".bold(), source);
        Ok(())
    }
}
