use crate::prelude::*;

/// List the sources in the queue
#[injectable]
pub(crate) struct QueueListCommand {
    shared_options: Ref<SharedOptions>,
    batch_options: Ref<BatchOptions>,
    queue: Ref<Queue>,
}

impl QueueListCommand {
    pub(crate) async fn execute_cli(&self) -> Result<bool, Error> {
        let transcode_enabled = self.batch_options.transcode;
        let retry_failed_transcodes = self.batch_options.retry_transcode;
        let upload_enabled = self.batch_options.upload;
        let indexer = self
            .shared_options
            .indexer
            .clone()
            .expect("indexer should be set");
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
        let found = items.len();
        info!(
            "{} {found} unprocessed sources in the queue for {}",
            "Found".bold(),
            indexer.to_uppercase()
        );
        let pad = found.to_string().len();
        let mut index = 1;
        for hash in items {
            let Some(item) = self.queue.get(hash)? else {
                error!("{} to retrieve {hash} from the queue", "Failed".bold());
                continue;
            };
            info!("{}: {item}", format!("{index:pad$}").bold());
            debug!("{}", item.path.display());
            debug!("{hash}");
            if let Some(id) = item.id {
                debug!("{id}");
            }
            index += 1;
        }
        Ok(true)
    }
}
