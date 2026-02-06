use crate::prelude::*;

/// List the sources in the queue
#[injectable]
pub(crate) struct QueueSummaryCommand {
    queue: Ref<Queue>,
}

impl QueueSummaryCommand {
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<QueueAction>> {
        let summary = self.execute().await?;
        let yaml = serde_yaml::to_string(&summary).expect("should be able to serialize summary");
        println!("{yaml}");
        Ok(true)
    }

    pub(crate) async fn execute(&self) -> Result<QueueSummary, Failure<QueueAction>> {
        let items = self.queue.get_all().await?;
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
            }
            match item.spectrogram {
                None => summary.spectrogram_none += 1,
                Some(SpectrogramStatus { success: true, .. }) => {
                    summary.spectrogram_success_true += 1;
                }
                Some(SpectrogramStatus { success: false, .. }) => {
                    summary.spectrogram_success_false += 1;
                }
            }
            match item.transcode {
                None => summary.transcode_none += 1,
                Some(TranscodeStatus { success: true, .. }) => summary.transcode_success_true += 1,
                Some(TranscodeStatus { success: false, .. }) => {
                    summary.transcode_success_false += 1;
                }
            }
            match item.upload {
                None => summary.upload_none += 1,
                Some(UploadStatus { success: true, .. }) => summary.upload_success_true += 1,
                Some(UploadStatus { success: false, .. }) => summary.upload_success_false += 1,
            }
        }
        Ok(summary)
    }
}
