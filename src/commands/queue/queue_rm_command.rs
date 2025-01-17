use crate::commands::*;
use crate::options::*;

use colored::Colorize;
use di::{injectable, Ref, RefMut};
use flat_db::Hash;
use log::{debug, info, warn};
use rogue_logging::Error;

/// Remove an item from the queue
#[injectable]
pub(crate) struct QueueRemoveCommand {
    shared_options: Ref<SharedOptions>,
    cache_options: Ref<CacheOptions>,
    args: Ref<QueueRemoveArgs>,
    queue: RefMut<Queue>,
}

impl QueueRemoveCommand {
    pub(crate) async fn execute_cli(&mut self) -> Result<bool, Error> {
        if !self.shared_options.validate()
            || !self.cache_options.validate()
            || !self.args.validate()
        {
            return Ok(false);
        }
        let hash = self
            .args
            .queue_rm_hash
            .clone()
            .expect("source should be set");
        let hash = Hash::from_string(&hash).expect("hash should be valid");
        let status = self.execute(hash).await?;
        Ok(status)
    }

    async fn execute(&mut self, hash: Hash<20>) -> Result<bool, Error> {
        let mut queue = self.queue.write().expect("queue should be writeable");
        debug!("{} item {hash} from queue", "Removing".bold());
        match queue.remove(hash).await? {
            None => {
                warn!(
                    "{} to remove {hash} from queue. Item does not exist",
                    "Failed".bold()
                );
                Ok(false)
            }
            Some(item) => {
                info!(
                    "{} item from queue: {}",
                    "Removed".bold(),
                    item.name.dimmed()
                );
                Ok(true)
            }
        }
    }
}
