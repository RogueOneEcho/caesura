use std::collections::BTreeMap;
use std::fs::create_dir;
use std::path::PathBuf;

use crate::commands::*;
use crate::dependencies::*;
use crate::options::*;

use di::{inject, injectable, Ref};
use flat_db::{Hash, Table};
use futures::stream::{iter, StreamExt};
use log::error;
use rogue_logging::Error;

/// Queue of FLAC sources and their statuses.
///
/// Each source is represented by a [`QueueItem`] stored by 20 byte SHA-1 hash.
///
/// Items are stored and retrieved as chunks by [`Table<20, 1, QueueItem>`].
///
/// Chunks are determined by taking the first byte of the hash.
///
/// As a byte represents 256 (2^8) values you can determine the approximate
/// number of sources per chunk with `total / 256` therefore:
///   `1,000` total ≈   `4` per chunk
///   `5,000` total ≈  `20` per chunk
///  `10,000` total ≈  `39` per chunk
///  `50,000` total ≈ `195` per chunk
/// `100,000` total ≈ `390` per chunk
#[injectable]
pub(crate) struct Queue {
    /// Path to the queue file
    table: Table<20, 1, QueueItem>,
}

#[allow(dead_code)]
impl Queue {
    /// Create a new [`Queue`]
    #[allow(dead_code)]
    pub(crate) fn from_path(path: PathBuf) -> Self {
        Self {
            table: Table::new(path),
        }
    }

    /// DI constructor for [`Queue`]
    #[inject]
    pub(crate) fn from_options(options: Ref<CacheOptions>) -> Self {
        let path = options.cache.clone().expect("queue path should be set");
        let path = path.join("queue");
        if !path.exists() {
            create_dir(&path)
                .expect("should be able to create queue directory if it does not exist");
        }
        Self::from_path(path)
    }

    /// Get an item from the queue
    pub(crate) fn get(&self, hash: Hash<20>) -> Result<Option<QueueItem>, Error> {
        self.table.get(hash)
    }

    /// Get the keys of the items that have not been processed.
    ///
    /// Items are filtered to ensure they have:
    /// - the correct indexer
    /// - not been verified, unless `transcode_enabled` is true
    /// - not been transcoded, unless `upload_enabled` is true
    /// - not been verified OR have been and `verified` is true
    /// - not been transcoded OR have been and `success` is true
    /// - not been uploaded
    ///
    /// Items are sorted by name
    pub(crate) async fn get_unprocessed(
        &mut self,
        indexer: String,
        transcode_enabled: bool,
        upload_enabled: bool,
        retry_failed_transcodes: bool,
    ) -> Result<Vec<Hash<20>>, Error> {
        let is_red = indexer == "red";
        let mut items = self
            .get_unprocessed_internal(
                indexer,
                transcode_enabled,
                upload_enabled,
                retry_failed_transcodes,
            )
            .await?;
        if is_red {
            let mut pth_items = self
                .get_unprocessed_internal(
                    "pth".to_owned(),
                    transcode_enabled,
                    upload_enabled,
                    retry_failed_transcodes,
                )
                .await?;
            items.append(&mut pth_items);
        }
        Ok(items)
    }

    /// Get the keys of the items that have not been processed.
    ///
    /// Items are filtered to ensure they have:
    /// - the correct indexer
    /// - not been verified, unless `transcode_enabled` is true
    /// - not been transcoded, unless `upload_enabled` is true
    /// - not been verified OR have been and `verified` is true
    /// - not been transcoded OR have been and `success` is true
    /// - not been uploaded
    ///
    /// Items are sorted by name
    async fn get_unprocessed_internal(
        &mut self,
        indexer: String,
        transcode_enabled: bool,
        upload_enabled: bool,
        retry_failed_transcodes: bool,
    ) -> Result<Vec<Hash<20>>, Error> {
        let items = self.table.get_all().await?;
        let mut items: Vec<&QueueItem> = items
            .values()
            .filter(|item| {
                item.indexer == indexer
                    && exclude_verified_if_transcode_disabled(item, transcode_enabled)
                    && exclude_transcoded_if_upload_disabled(item, upload_enabled)
                    && exclude_verify_failures(item)
                    && exclude_transcode_failures(item, retry_failed_transcodes)
                    && item.upload.is_none()
            })
            .collect();
        items.sort_by_key(|x| &x.name);
        let hashes = items.iter().map(|x| x.hash).collect();
        Ok(hashes)
    }

    /// Get all items.
    ///
    /// Items are unsorted.
    pub(crate) async fn get_all(&mut self) -> Result<BTreeMap<Hash<20>, QueueItem>, Error> {
        self.table.get_all().await
    }

    /// Update an item into the queue
    pub(crate) async fn set(&mut self, item: QueueItem) -> Result<(), Error> {
        self.table.set(item.hash, item).await
    }

    /// Add many items.
    ///
    /// If `replace` is true then existing items are replaced
    ///
    /// Items are chunked together to minimize IO operations.
    ///
    /// Returns the number of items added
    pub(crate) async fn set_many(
        &self,
        items: BTreeMap<Hash<20>, QueueItem>,
        replace: bool,
    ) -> Result<usize, Error> {
        self.table.set_many(items, replace).await
    }

    /// Insert torrent files into the queue if they are not already present
    /// Returns the number of items added
    pub(crate) async fn insert_new_torrent_files(
        &mut self,
        paths: Vec<PathBuf>,
    ) -> Result<usize, Error> {
        let stream = iter(paths.into_iter());
        let items: BTreeMap<_, _> = stream
            .filter_map(|path| async {
                let torrent = match ImdlCommand::show(&path).await {
                    Ok(torrent) => Some(torrent),
                    Err(error) => {
                        error!("Failed to read torrent: {}\n{error}", path.display());
                        None
                    }
                };
                let item = QueueItem::from_torrent(path, torrent?);
                Some((item.hash, item))
            })
            .collect()
            .await;
        self.table.set_many(items, false).await
    }
}

fn exclude_verify_failures(item: &QueueItem) -> bool {
    !matches!(
        item.verify,
        Some(VerifyStatus {
            verified: false,
            ..
        })
    )
}

fn exclude_transcode_failures(item: &QueueItem, retry_failed_transcodes: bool) -> bool {
    retry_failed_transcodes
        || !matches!(item.transcode, Some(TranscodeStatus { success: false, .. }))
}

fn exclude_verified_if_transcode_disabled(item: &QueueItem, transcode_enabled: bool) -> bool {
    transcode_enabled || item.verify.is_none()
}

fn exclude_transcoded_if_upload_disabled(item: &QueueItem, upload_enabled: bool) -> bool {
    upload_enabled || !matches!(item.transcode, Some(TranscodeStatus { success: true, .. }))
}
