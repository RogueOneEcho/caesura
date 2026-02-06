use crate::prelude::*;
use gazelle_api::{ApiResponseKind, GazelleError, GazelleOperation};
use std::error::Error;
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
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<BatchAction>> {
        let spectrogram_enabled = self.batch_options.spectrogram;
        let transcode_enabled = self.batch_options.transcode;
        let retry_failed_transcodes = self.batch_options.retry_transcode;
        let upload_enabled = self.batch_options.upload;
        let indexer = self.shared_options.indexer.clone();
        let limit = self.batch_options.get_limit();
        let items = self
            .queue
            .get_unprocessed(
                indexer.clone(),
                transcode_enabled,
                upload_enabled,
                retry_failed_transcodes,
            )
            .await
            .map_err(Failure::wrap(BatchAction::GetUnprocessed))?;
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
            let Some(mut item) = self
                .queue
                .get(hash)
                .await
                .map_err(Failure::wrap(BatchAction::GetQueueItem))?
            else {
                error!("{} to retrieve {hash} from the queue", "Failed".bold());
                continue;
            };
            trace!("{} {item}", "Processing".bold());
            let Some(id) = item.id else {
                debug!("{} {item} as it doesn't have an id", "Skipping".bold());
                let status = VerifyStatus::from_issue(SourceIssue::Id(IdProviderError::NoId));
                item.verify = Some(status);
                self.queue
                    .set(item)
                    .await
                    .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
                continue;
            };
            let source = match self.source_provider.get(id).await {
                Ok(Ok(source)) => source,
                Ok(Err(issue)) => {
                    debug!("{} {item}", "Skipping".bold());
                    debug!("{issue}");
                    item.verify = Some(VerifyStatus::from_issue(issue));
                    self.queue
                        .set(item)
                        .await
                        .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
                    continue;
                }
                Err(failure) => {
                    debug!("{} {item}", "Skipping".bold());
                    debug!("{failure}");
                    let api_error = failure
                        .source()
                        .and_then(|e| e.downcast_ref::<GazelleError>());
                    match api_error.map(|e| &e.operation) {
                        Some(GazelleOperation::ApiResponse(ApiResponseKind::Unauthorized)) => {
                            return Err(Failure::new(
                                BatchAction::GetSource,
                                BatchError::Unauthorized,
                            ));
                        }
                        Some(GazelleOperation::ApiResponse(ApiResponseKind::TooManyRequests)) => {
                            warn!("{} rate limit", "Exceeded".bold());
                            pause().await;
                            continue;
                        }
                        Some(GazelleOperation::ApiResponse(ApiResponseKind::Other)) => {
                            warn!("{} response received", "Unexpected".bold());
                            warn!("{}", failure.render());
                            pause().await;
                            continue;
                        }
                        _ => {
                            warn!("{}", failure.render());
                            continue;
                        }
                    }
                }
            };
            let result = self.verify.execute(&source).await;
            let verified = result.verified();
            if verified {
                debug!("{} {}", "Verified".bold(), source);
            } else {
                debug!("{} {source}", "Skipping".bold());
                debug!("{} for transcoding {}", "Unsuitable".bold(), source);
                for issue in &result.issues {
                    debug!("{issue}");
                }
            }
            item.verify = Some(VerifyStatus::new(Ok(result)));
            if !verified {
                self.queue
                    .set(item)
                    .await
                    .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
                continue;
            }
            if spectrogram_enabled {
                let result = self.spectrogram.execute(&source).await;
                if let Err(e) = &result {
                    warn!("{}", e.render());
                }
                item.spectrogram = Some(SpectrogramStatus::new(result));
            }
            if transcode_enabled {
                let result = self.transcode.execute(&source).await;
                let success = result.is_ok();
                if let Err(e) = &result {
                    error!("{}", e.render());
                }
                let status = TranscodeStatus::new(result);
                item.transcode = Some(status);
                if !success {
                    self.queue
                        .set(item)
                        .await
                        .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
                    continue;
                }
                if upload_enabled {
                    if let Some(wait_before_upload) = self.batch_options.get_wait_before_upload() {
                        info!("{} {wait_before_upload:?} before upload", "Waiting".bold());
                        sleep(wait_before_upload).await;
                    }
                    let result = self.upload.execute(&source).await;
                    if let Err(e) = &result {
                        error!("{}", e.render());
                    }
                    if !self.upload_options.dry_run {
                        item.upload = Some(UploadStatus::new(result));
                    }
                }
            }
            self.queue
                .set(item)
                .await
                .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
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
