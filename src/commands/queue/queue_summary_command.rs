use crate::commands::*;
use crate::options::*;
use crate::utils::*;

use di::{injectable, Ref, RefMut};
use rogue_logging::Error;

/// List the sources in the queue
#[injectable]
pub(crate) struct QueueSummaryCommand {
    cache_options: Ref<CacheOptions>,
    queue: RefMut<Queue>,
}

impl QueueSummaryCommand {
    pub(crate) async fn execute_cli(&mut self) -> Result<bool, Error> {
        if !self.cache_options.validate() {
            return Ok(false);
        }
        let summary = self.execute().await?;
        let yaml = serde_yaml::to_string(&summary)
            .map_err(|e| yaml_error(e, "serialize queue summary"))?;
        println!("{yaml}");
        Ok(true)
    }

    pub(crate) async fn execute(&mut self) -> Result<QueueSummary, Error> {
        let mut queue = self.queue.write().expect("Queue should be writeable");
        let items = queue.get_all().await?;
        let mut summary = QueueSummary::default();
        for (_, item) in items {
            summary.total += 1;
            match summary.indexer.get_mut(&item.indexer) {
                Some(count) => *count += 1,
                None => {
                    summary.indexer.insert(item.indexer.clone(), 1);
                }
            }
            match item.verify {
                None => summary.verify_none += 1,
                Some(VerifyStatus { verified: true, .. }) => summary.verify_verified_true += 1,
                Some(VerifyStatus {
                    verified: false, ..
                }) => summary.verify_verified_false += 1,
            };
            match item.spectrogram {
                None => summary.spectrogram_none += 1,
                Some(SpectrogramStatus { success: true, .. }) => {
                    summary.spectrogram_success_true += 1;
                }
                Some(SpectrogramStatus { success: false, .. }) => {
                    summary.spectrogram_success_false += 1;
                }
            };
            match item.transcode {
                None => summary.transcode_none += 1,
                Some(TranscodeStatus { success: true, .. }) => summary.transcode_success_true += 1,
                Some(TranscodeStatus { success: false, .. }) => {
                    summary.transcode_success_false += 1;
                }
            };
            match item.upload {
                None => summary.upload_none += 1,
                Some(UploadStatus { success: true, .. }) => summary.upload_success_true += 1,
                Some(UploadStatus { success: false, .. }) => summary.upload_success_false += 1,
            };
        }
        Ok(summary)
    }
}
