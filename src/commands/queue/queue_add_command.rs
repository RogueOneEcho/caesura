use crate::commands::*;
use crate::options::*;
use crate::utils::*;

use colored::Colorize;
use di::{Ref, injectable};
use flat_db::Hash;
use log::{info, trace};
use rogue_logging::Error;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Add a directory of `.torrent` files to the queue
#[injectable]
pub(crate) struct QueueAddCommand {
    shared_options: Ref<SharedOptions>,
    cache_options: Ref<CacheOptions>,
    args: Ref<QueueAddArgs>,
    queue: Ref<Queue>,
}

impl QueueAddCommand {
    pub(crate) async fn execute_cli(&self) -> Result<bool, Error> {
        if !self.shared_options.validate()
            || !self.cache_options.validate()
            || !self.args.validate()
        {
            return Ok(false);
        }
        let path = self
            .args
            .queue_add_path
            .clone()
            .expect("source should be set");
        let status = self.execute(path).await?;
        info!("{} {} items to the queue", "Added".bold(), status.added);
        trace!(
            "{} {} items already in the queue",
            "Excluded".bold(),
            status.excluded
        );
        Ok(true)
    }

    async fn execute(&self, path: PathBuf) -> Result<QueueStatus, Error> {
        if path.is_dir() {
            self.execute_directory(path).await
        } else if path.is_file() {
            self.execute_file(path).await
        } else {
            Err(error(
                "add to queue",
                format!("Does not exist: {}", path.display()),
            ))
        }
    }

    async fn execute_directory(&self, path: PathBuf) -> Result<QueueStatus, Error> {
        let existing_paths: Vec<PathBuf> = self
            .queue
            .get_all()
            .await?
            .values()
            .map(|x| x.path.clone())
            .collect();
        trace!(
            "{} {} existing paths",
            "Skipping".bold(),
            existing_paths.len()
        );
        trace!("Reading torrent directory: {}", path.display());
        let paths = DirectoryReader::new()
            .with_extension("torrent")
            .with_max_depth(0)
            .read(&path)
            .map_err(|e| io_error(e, "read torrent directory"))?;
        let found = paths.len();
        trace!("{} {} torrent files", "Found".bold(), found);
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .filter(|x| !existing_paths.contains(x))
            .collect();
        let remaining = paths.len();
        info!("{} {} new torrent files", "Found".bold(), remaining);
        if remaining > 250 {
            info!("This may take a while");
        }
        let added = self.queue.insert_new_torrent_files(paths).await?;
        Ok(QueueStatus {
            success: true,
            added,
            excluded: found - added,
        })
    }

    async fn execute_file(&self, path: PathBuf) -> Result<QueueStatus, Error> {
        trace!("Reading queue file: {}", path.display());
        let file = File::open(path).map_err(|e| io_error(e, "open chunk file"))?;
        let reader = BufReader::new(file);
        let items: BTreeMap<Hash<20>, QueueItem> =
            serde_yaml::from_reader(reader).map_err(|e| yaml_error(e, "deserialize chunk"))?;
        let found = items.len();
        info!("{} {} items", "Found".bold(), found);
        if found > 250 {
            info!("This may take a while");
        }
        let added = self.queue.set_many(items, true).await?;
        Ok(QueueStatus {
            success: true,
            added,
            excluded: found - added,
        })
    }
}
