use crate::prelude::*;
use gazelle_api::GazelleError;
use std::time::Duration;
use tokio::time::sleep;

const PAUSE_DURATION: u64 = 10;

/// Process multiple sources from the queue.
#[injectable]
pub(crate) struct BatchCommand {
    shared_options: Ref<SharedOptions>,
    upload_options: Ref<UploadOptions>,
    batch_options: Ref<BatchOptions>,
    source_provider: Ref<SourceProvider>,
    verify: Ref<VerifyCommand>,
    spectrogram: Ref<SpectrogramCommand>,
    transcode: Ref<TranscodeCommand>,
    upload: Ref<UploadCommand>,
    queue: Ref<Queue>,
}

impl BatchCommand {
    /// Execute [`BatchCommand`] from the CLI.
    ///
    /// Returns `true` if the batch process succeeds.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn execute_cli(&self) -> Result<bool, Error> {
        let spectrogram_enabled = self.batch_options.spectrogram;
        let transcode_enabled = self.batch_options.transcode;
        let retry_failed_transcodes = self.batch_options.retry_transcode;
        let upload_enabled = self.batch_options.upload;
        let indexer = self
            .shared_options
            .indexer
            .clone()
            .expect("indexer should be set");
        let limit = self.batch_options.get_limit();
        let items = self
            .queue
            .get_unprocessed(
                indexer.clone(),
                transcode_enabled,
                upload_enabled,
                retry_failed_transcodes,
            )
            .await?;
        if items.is_empty() {
            info!(
                "{} items in the queue for {}",
                "No".bold(),
                indexer.to_uppercase()
            );
            info!("{} the `queue` command to add items", "Use".bold());
            return Ok(true);
        }
        debug!(
            "{} {} sources in the queue for {}",
            "Found".bold(),
            items.len(),
            indexer.to_uppercase()
        );
        let mut count = 0;
        for hash in items {
            let Some(mut item) = self.queue.get(hash)? else {
                error!("{} to retrieve {hash} from the queue", "Failed".bold());
                continue;
            };
            trace!("{} {item}", "Processing".bold());
            let Some(id) = item.id else {
                debug!("{} {item} as it doesn't have an id", "Skipping".bold());
                let status = VerifyStatus::from_issue(SourceIssue::Id(IdProviderError::NoId));
                item.verify = Some(status);
                self.queue.set(item).await?;
                continue;
            };
            let source = match self.source_provider.get(id).await {
                Ok(source) => source,
                Err(issue) => {
                    debug!("{} {item}", "Skipping".bold());
                    debug!("{issue}");
                    match issue.clone() {
                        SourceIssue::Api {
                            response: GazelleError::Unauthorized { message: _ },
                        } => {
                            trace!("{issue}");
                            return Err(error(
                                "get source",
                                format!(
                                    "{} response received. This likely means the API Key is invalid.",
                                    "Unauthorized".bold()
                                ),
                            ));
                        }
                        SourceIssue::Api {
                            response: GazelleError::TooManyRequests { message: _ },
                        } => {
                            trace!("{issue}");
                            warn!("{} rate limit", "Exceeded".bold());
                            pause().await;
                        }
                        SourceIssue::Api {
                            response:
                                GazelleError::Other {
                                    status: _,
                                    message: _,
                                },
                        } => {
                            warn!("{} response received", "Unexpected".bold());
                            warn!("{issue}");
                            pause().await;
                        }
                        _ => {
                            item.verify = Some(VerifyStatus::from_issue(issue));
                            self.queue.set(item).await?;
                        }
                    }
                    continue;
                }
            };
            let status = self.verify.execute(&source).await;
            if status.verified {
                debug!("{} {}", "Verified".bold(), source);
                item.verify = Some(status);
            } else {
                debug!("{} {source}", "Skipping".bold());
                debug!("{} for transcoding {}", "Unsuitable".bold(), source);
                if let Some(issues) = &status.issues {
                    for issue in issues {
                        debug!("{issue}");
                    }
                }
                item.verify = Some(status);
                self.queue.set(item).await?;
                continue;
            }
            if spectrogram_enabled {
                let status = self.spectrogram.execute(&source).await;
                if let Some(error) = &status.error {
                    warn!("{error}");
                }
                item.spectrogram = Some(status);
            }
            if transcode_enabled {
                let status = self.transcode.execute(&source).await;
                if let Some(error) = &status.error {
                    error.log();
                }
                if status.success {
                    item.transcode = Some(status);
                } else {
                    item.transcode = Some(status);
                    self.queue.set(item).await?;
                    continue;
                }
                if upload_enabled {
                    if let Some(wait_before_upload) = self.batch_options.get_wait_before_upload() {
                        info!("{} {wait_before_upload:?} before upload", "Waiting".bold());
                        sleep(wait_before_upload).await;
                    }
                    let status = self.upload.execute(&source).await;
                    if !self.upload_options.dry_run {
                        item.upload = Some(status);
                    }
                    // Errors were already logged in UploadCommand::Execute()
                }
            }
            self.queue.set(item).await?;
            count += 1;
            if let Some(limit) = limit
                && count >= limit
            {
                info!("{} batch limit: {limit}", "Reached".bold());
                break;
            }
        }
        info!("{} batch process of {count} items", "Completed".bold());
        Ok(true)
    }
}

async fn pause() {
    info!("There is no retry logic so you will need to re-run the command");
    info!("If it persists, please submit an issue on GitHub.");
    info!("{} for {PAUSE_DURATION} seconds.", "Pausing".bold());
    sleep(Duration::from_secs(PAUSE_DURATION)).await;
}
