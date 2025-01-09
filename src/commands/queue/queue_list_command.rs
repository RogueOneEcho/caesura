use crate::commands::*;
use crate::options::*;

use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{debug, error, info};
use rogue_logging::Error;

/// List the sources in the queue
#[injectable]
pub(crate) struct QueueListCommand {
    shared_options: Ref<SharedOptions>,
    cache_options: Ref<CacheOptions>,
    batch_options: Ref<BatchOptions>,
    queue: RefMut<Queue>,
}

impl QueueListCommand {
    pub(crate) async fn execute_cli(&mut self) -> Result<bool, Error> {
        if !self.shared_options.validate()
            || !self.cache_options.validate()
            || !self.batch_options.validate()
        {
            return Ok(false);
        }
        let mut queue = self.queue.write().expect("Queue should be writeable");
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
        let found = items.len();
        info!(
            "{} {found} unprocessed sources in the queue for {}",
            "Found".bold(),
            indexer.to_uppercase()
        );
        let pad = found.to_string().len();
        let mut index = 1;
        for hash in items {
            let Some(item) = queue.get(hash)? else {
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
