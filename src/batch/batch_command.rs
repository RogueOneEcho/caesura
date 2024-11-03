use crate::options::{
    BatchOptions, CacheOptions, FileOptions, Options, SharedOptions, SpectrogramOptions,
    TargetOptions, VerifyOptions,
};
use crate::queue::Queue;
use crate::source::*;
use crate::spectrogram::SpectrogramCommand;
use crate::transcode::TranscodeCommand;
use crate::upload::UploadCommand;
use crate::verify::{VerifyCommand, VerifyStatus};
use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{debug, error, info, trace, warn};
use reqwest::StatusCode;
use rogue_logging::Error;
use tokio::time::sleep;

/// Batch a FLAC source is suitable for transcoding.
#[injectable]
pub struct BatchCommand {
    cache_options: Ref<CacheOptions>,
    shared_options: Ref<SharedOptions>,
    verify_options: Ref<VerifyOptions>,
    target_options: Ref<TargetOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    file_options: Ref<FileOptions>,
    batch_options: Ref<BatchOptions>,
    source_provider: RefMut<SourceProvider>,
    verify: RefMut<VerifyCommand>,
    spectrogram: Ref<SpectrogramCommand>,
    transcode: Ref<TranscodeCommand>,
    upload: RefMut<UploadCommand>,
    queue: RefMut<Queue>,
}

impl BatchCommand {
    /// Execute [`BatchCommand`] from the CLI.
    ///
    /// Returns `true` if the batch process succeeds.
    #[allow(clippy::too_many_lines)]
    pub async fn execute_cli(&mut self) -> Result<bool, Error> {
        if !self.cache_options.validate()
            || !self.shared_options.validate()
            || !self.verify_options.validate()
            || !self.target_options.validate()
            || !self.spectrogram_options.validate()
            || !self.file_options.validate()
            || !self.batch_options.validate()
        {
            return Ok(false);
        }
        let mut queue = self.queue.write().expect("Queue should be writeable");
        let mut source_provider = self
            .source_provider
            .write()
            .expect("SourceProvider should be writable");
        let spectrogram_enabled = self
            .batch_options
            .spectrogram
            .expect("spectrogram should be set");
        let transcode_enabled = self
            .batch_options
            .transcode
            .expect("transcode should be set");
        let retry_failed_transcodes = self
            .batch_options
            .retry_transcode
            .expect("retry_transcode should be set");
        let upload_enabled = self.batch_options.upload.expect("upload should be set");
        let indexer = self
            .shared_options
            .indexer
            .clone()
            .expect("indexer should be set");
        let limit = self.batch_options.get_limit();
        let items = queue
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
            let Some(mut item) = queue.get(hash)? else {
                error!("{} to retrieve {hash} from the queue", "Failed".bold());
                continue;
            };
            trace!("{} {item}", "Processing".bold());
            let Some(id) = item.id else {
                debug!("{} {item} as it doesn't have an id", "Skipping".bold());
                let status = VerifyStatus::from_issue(SourceIssue::IdError {
                    details: "missing id".to_owned(),
                });
                item.verify = Some(status);
                queue.set(item).await?;
                continue;
            };
            let source = match source_provider.get(id).await {
                Ok(source) => source,
                Err(issue) => {
                    if let SourceIssue::ApiResponse {
                        action: _,
                        status_code,
                        error,
                    } = issue.clone()
                    {
                        let reason = StatusCode::from_u16(status_code).map_or_else(
                            |_| status_code.to_string(),
                            |sc| sc.canonical_reason().unwrap_or("").to_owned(),
                        );
                        if status_code == 429 || status_code >= 500 {
                            warn!("{} {item} due to {reason}", "Skipping".bold());
                            warn!("{error}");
                            warn!("This is likely to be a temporary issue with the API.");
                            warn!("If it persists, please submit an issue on GitHub.");
                        } else {
                            debug!("{} {item} due to {reason}", "Skipping".bold());
                            debug!("{error}");
                            item.verify = Some(VerifyStatus::from_issue(issue));
                            queue.set(item).await?;
                        }
                    } else {
                        debug!("{} {item}", "Skipping".bold());
                        debug!("{issue}");
                        item.verify = Some(VerifyStatus::from_issue(issue));
                        queue.set(item).await?;
                    }
                    continue;
                }
            };
            let status = self
                .verify
                .write()
                .expect("VerifyCommand should be writeable")
                .execute(&source)
                .await;
            if status.verified {
                debug!("{} {}", "Verified".bold(), source);
                item.verify = Some(status);
            } else {
                debug!("{} {source}", "Skipping".bold());
                debug!("{} to verify {}", "Failed".bold(), source);
                if let Some(issues) = &status.issues {
                    for issue in issues {
                        debug!("{issue}");
                    }
                }
                item.verify = Some(status);
                queue.set(item).await?;
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
                    queue.set(item).await?;
                    continue;
                }
                if upload_enabled {
                    if let Some(wait_before_upload) = self.batch_options.get_wait_before_upload() {
                        info!("{} {wait_before_upload:?} before upload", "Waiting".bold());
                        sleep(wait_before_upload).await;
                    }
                    let status = self
                        .upload
                        .write()
                        .expect("UploadCommand should be writeable")
                        .execute(&source)
                        .await;
                    // Errors were already logged in UploadCommand::Execute()
                    item.upload = Some(status);
                }
            }
            queue.set(item).await?;
            count += 1;
            if let Some(limit) = limit {
                if count >= limit {
                    info!("{} batch limit: {limit}", "Reached".bold());
                    break;
                }
            }
        }
        info!("{} batch process of {count} items", "Completed".bold());
        Ok(true)
    }
}
