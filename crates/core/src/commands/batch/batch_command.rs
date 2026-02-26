use crate::prelude::*;
use gazelle_api::{ApiResponseKind, GazelleError, GazelleOperation};
use serde::Serialize;
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
    paths: Ref<PathManager>,
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
        let dry_run = upload_enabled && self.upload_options.dry_run;
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
                let transcode_result = self.transcode.execute(&source).await;
                let transcode_formats = transcode_result
                    .as_ref()
                    .ok()
                    .map(|success| success.formats.clone())
                    .unwrap_or_default();
                let transcode_success = transcode_result.is_ok();
                if let Err(e) = &transcode_result {
                    error!("{}", e.render());
                }
                let status = TranscodeStatus::new(transcode_result);
                item.transcode = Some(status);
                if !transcode_success {
                    self.queue
                        .set(item)
                        .await
                        .map_err(Failure::wrap(BatchAction::UpdateQueueItem))?;
                    continue;
                }
                if let Some(post_transcode_hook) = &self.batch_options.post_transcode_hook {
                    if dry_run {
                        info!(
                            "{} post-transcode hook as upload dry run is enabled",
                            "Skipping".bold()
                        );
                    } else {
                        for format in &transcode_formats {
                            let torrent_path = self.paths.get_torrent_path(&source, format.format);
                            let payload = self.create_hook_payload(
                                &source,
                                &format.path,
                                &torrent_path,
                                None,
                            );
                            if let Err(e) = self.execute_hook(post_transcode_hook, &payload).await {
                                warn!("{}", e.render());
                            }
                        }
                    }
                }
                if upload_enabled {
                    if let Some(wait_before_upload) = self.batch_options.get_wait_before_upload() {
                        info!("{} {wait_before_upload:?} before upload", "Waiting".bold());
                        sleep(wait_before_upload).await;
                    }
                    let upload_result = self.upload.execute(&source).await;
                    let upload_formats = upload_result
                        .as_ref()
                        .ok()
                        .map(|success| success.formats.clone())
                        .unwrap_or_default();
                    if let Err(e) = &upload_result {
                        error!("{}", e.render());
                    }
                    if let Some(post_upload_hook) = &self.batch_options.post_upload_hook {
                        if dry_run {
                            info!(
                                "{} post-upload hook as upload dry run is enabled",
                                "Skipping".bold()
                            );
                        } else {
                            for format in upload_formats {
                                let transcode_path =
                                    self.paths.get_transcode_target_dir(&source, format.format);
                                let torrent_path =
                                    self.paths.get_torrent_path(&source, format.format);
                                let payload = self.create_hook_payload(
                                    &source,
                                    &transcode_path,
                                    &torrent_path,
                                    Some(format.id),
                                );
                                if let Err(e) = self.execute_hook(post_upload_hook, &payload).await
                                {
                                    warn!("{}", e.render());
                                }
                            }
                        }
                    }
                    if !dry_run {
                        item.upload = Some(UploadStatus::new(upload_result));
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

    fn create_hook_payload(
        &self,
        source: &Source,
        transcode_path: &Path,
        torrent_path: &Path,
        torrent_id: Option<u32>,
    ) -> BatchHookPayload {
        let permalink = torrent_id
            .map(|id| get_permalink(&self.shared_options.indexer_url, source.group.id, id));
        BatchHookPayload {
            torrent_id,
            group_id: Some(source.group.id),
            permalink,
            source_name: SourceName::get(&source.metadata),
            source_path: source.directory.to_string_lossy().to_string(),
            transcode_path: transcode_path.to_string_lossy().to_string(),
            torrent_path: torrent_path.to_string_lossy().to_string(),
        }
    }

    async fn execute_hook(
        &self,
        hook_path: &Path,
        payload: &BatchHookPayload,
    ) -> Result<(), Failure<BatchAction>> {
        execute_yaml_hook(
            hook_path,
            payload,
            BatchAction::SerializeHookPayload,
            BatchAction::WriteHookPayload,
            BatchAction::ExecuteHook,
        )
        .await
    }
}

#[derive(Serialize)]
struct BatchHookPayload {
    torrent_id: Option<u32>,
    group_id: Option<u32>,
    permalink: Option<String>,
    source_name: String,
    source_path: String,
    transcode_path: String,
    torrent_path: String,
}

async fn pause() {
    info!("There is no retry logic so you will need to re-run the command");
    info!("If it persists, please submit an issue on GitHub.");
    info!("{} for {PAUSE_DURATION} seconds.", "Pausing".bold());
    sleep(Duration::from_secs(PAUSE_DURATION)).await;
}
