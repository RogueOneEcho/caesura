use crate::prelude::*;
use flat_db::Hash;

/// Remove an item from the queue
#[injectable]
pub(crate) struct QueueRemoveCommand {
    args: Ref<QueueRemoveArgs>,
    queue: Ref<Queue>,
}

impl QueueRemoveCommand {
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<QueueAction>> {
        if !self.args.validate() {
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

    async fn execute(&self, hash: Hash<20>) -> Result<bool, Failure<QueueAction>> {
        debug!("{} item {hash} from queue", "Removing".bold());
        match self.queue.remove(hash).await? {
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
