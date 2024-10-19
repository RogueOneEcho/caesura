use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{debug, info, trace};
use std::fmt::Write;

use crate::batch::BatchCacheItem;
use crate::batch::{BatchCache, BatchCacheFactory};
use crate::errors::AppError;
use crate::options::{
    BatchOptions, FileOptions, Options, SharedOptions, SpectrogramOptions, TargetOptions,
    VerifyOptions,
};
use crate::source::*;
use crate::spectrogram::SpectrogramCommand;
use crate::transcode::TranscodeCommand;
use crate::upload::UploadCommand;
use crate::verify::VerifyCommand;

/// Batch a FLAC source is suitable for transcoding.
#[injectable]
pub struct BatchCommand {
    shared_options: Ref<SharedOptions>,
    verify_options: Ref<VerifyOptions>,
    target_options: Ref<TargetOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    file_options: Ref<FileOptions>,
    batch_options: Ref<BatchOptions>,
    batch_cache_factory: RefMut<BatchCacheFactory>,
    id_provider: Ref<IdProvider>,
    source_provider: RefMut<SourceProvider>,
    verify: RefMut<VerifyCommand>,
    spectrogram: Ref<SpectrogramCommand>,
    transcode: Ref<TranscodeCommand>,
    upload: RefMut<UploadCommand>,
}

impl BatchCommand {
    pub async fn execute_cli(&mut self) -> Result<bool, AppError> {
        if !self.shared_options.validate()
            || !self.verify_options.validate()
            || !self.target_options.validate()
            || !self.spectrogram_options.validate()
            || !self.file_options.validate()
            || !self.batch_options.validate()
        {
            return Ok(false);
        }
        let mut cache = self
            .batch_cache_factory
            .write()
            .expect("BatchCacheFactory should be writeable")
            .create()?;
        let skip_spectrogram = !self
            .batch_options
            .spectrogram
            .expect("spectrogram should be set");
        let skip_upload = !self.batch_options.upload.expect("upload should be set");
        let queue = cache.get_queue(skip_upload);
        let limit = self.batch_options.get_limit();
        debug!("{} {} sources", "Queued".bold(), queue.len());
        let mut count = 0;
        for item in queue {
            let id = match self.id_provider.get_by_file(&item.path).await {
                Ok(id) => id,
                Err(error) => {
                    cache.update(&item.path, |item| item.set_skipped(error.to_string()));
                    debug!("{} {item}", "Skipping".bold());
                    trace!("{error}");
                    continue;
                }
            };
            let source = match self.get_source(id).await {
                Ok(source) => source,
                Err(error) => {
                    cache.update(&item.path, |item| item.set_skipped(error.to_string()));
                    debug!("{} {item}", "Skipping".bold());
                    trace!("{error}");
                    continue;
                }
            };
            if !self.verify(&source, &mut cache, &item).await {
                continue;
            }
            if !skip_spectrogram {
                self.spectrogram.execute(&source).await?;
            }
            if self.transcode.execute(&source).await? {
                cache.update(&item.path, BatchCacheItem::set_transcoded);
            } else {
                cache.update(&item.path, |item| {
                    item.set_failed("transcode failed".to_owned());
                });
                continue;
            }
            if !skip_upload {
                if let Some(wait_before_upload) = self.batch_options.get_wait_before_upload() {
                    info!("{} {wait_before_upload:?} before upload", "Waiting".bold());
                    tokio::time::sleep(wait_before_upload).await;
                }
                if self
                    .upload
                    .write()
                    .expect("UploadCommand should be writeable")
                    .execute(&source)
                    .await?
                {
                    cache.update(&item.path, BatchCacheItem::set_uploaded);
                } else {
                    cache.update(&item.path, |item| {
                        item.set_failed("upload failed".to_owned());
                    });
                    continue;
                }
            }
            cache.save(false)?;
            count += 1;
            if let Some(limit) = limit {
                if count >= limit {
                    info!("{} batch limit: {limit}", "Reached".bold());
                    break;
                }
            }
        }
        cache.save(true)?;
        info!("{} batch process of {count} items", "Completed".bold());
        Ok(true)
    }

    async fn get_source(&mut self, id: i64) -> Result<Source, AppError> {
        self.source_provider
            .write()
            .expect("SourceProvider should be writable")
            .get(id)
            .await
    }

    async fn verify(
        &mut self,
        source: &Source,
        cache: &mut BatchCache,
        item: &BatchCacheItem,
    ) -> bool {
        let status = self
            .verify
            .write()
            .expect("VerifyCommand should be writeable")
            .execute(source)
            .await;
        if status.verified {
            return true;
        }
        debug!("{} {source}", "Skipping".bold());
        let reason = status
            .violations
            .into_iter()
            .fold(String::new(), |mut buffer, violation| {
                writeln!(buffer, "{violation}").expect("should be able to use string as a buffer");
                buffer
            });
        trace!("{reason}");
        // TODO: Update cache to accept the reason
        cache.update(&item.path, |item| item.set_skipped(reason.clone()));
        false
    }
}
