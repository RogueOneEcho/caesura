use crate::prelude::*;
use std::io::BufReader;

/// Add a directory of `.torrent` files to the queue
#[injectable]
pub(crate) struct QueueAddCommand {
    args: Ref<QueueAddArgs>,
    queue: Ref<Queue>,
}

impl QueueAddCommand {
    /// Add torrent files from the configured path to the queue.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<QueueAction>> {
        let path = self
            .args
            .queue_add_path
            .clone()
            .expect("queue add path should be set after validation");
        let status = self.execute(path).await?;
        info!("{} {} items to the queue", "Added".bold(), status.added);
        trace!(
            "{} {} items already in the queue",
            "Excluded".bold(),
            status.excluded
        );
        Ok(true)
    }

    async fn execute(&self, path: PathBuf) -> Result<QueueStatus, Failure<QueueAction>> {
        if path.is_dir() {
            self.execute_directory(path).await
        } else if path.is_file() {
            self.execute_file(path).await
        } else {
            Err(Failure::new(
                QueueAction::MatchPath,
                IoError::new(ErrorKind::NotFound, "path does not exist"),
            )
            .with_path(&path))
        }
    }

    /// Discover torrents by scanning a directory and add new ones to the queue.
    ///
    /// - **List all `.torrent` files in the directory (bottleneck for large directories)**
    /// - Exclude paths already in the queue (in-memory, fast)
    /// - **Parse each new `.torrent` file with `lava_torrent` (bottleneck)**
    /// - Insert parsed items into the queue
    ///
    /// On a fresh queue, every `.torrent` file in the directory is parsed regardless
    /// of whether it belongs to a relevant category.
    /// On subsequent runs, only newly added files are parsed.
    async fn execute_directory(&self, path: PathBuf) -> Result<QueueStatus, Failure<QueueAction>> {
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
            .map_err(Failure::wrap_with_path(QueueAction::ReadTorrent, &path))?;
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

    /// Load a pre-built YAML queue file and add items to the queue.
    ///
    /// - Read and deserialize the YAML file (single file I/O, fast)
    /// - **Insert all items into the queue (bottleneck for large files)**
    async fn execute_file(&self, path: PathBuf) -> Result<QueueStatus, Failure<QueueAction>> {
        trace!("Reading queue file: {}", path.display());
        let file =
            File::open(&path).map_err(Failure::wrap_with_path(QueueAction::ReadTorrent, &path))?;
        let reader = BufReader::new(file);
        let items: BTreeMap<Hash<20>, QueueItem> = yaml_from_reader(reader)
            .map_err(Failure::wrap_with_path(QueueAction::ReadTorrent, &path))?;
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
