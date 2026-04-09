use crate::prelude::*;
use flat_db::Hash;
use qbittorrent_api::QBittorrentClientTrait;
use qbittorrent_api::get_torrents::{FilterOptions, Torrent};
use std::collections::HashSet;

/// Discover torrents via the qBittorrent API and add them to the queue.
///
/// - Query the API once per category (HTTP call, fast)
/// - Filter to fully downloaded torrents (in-memory, fast)
/// - Dedupe by hash and exclude torrents already in the queue (in-memory, fast)
/// - Build queue items directly from API response data (in-memory, fast)
///
/// On subsequent runs, only newly added torrents are inserted.
#[injectable]
pub(crate) struct QueueFetchCommand {
    qbit_options: Ref<QbitOptions>,
    queue_fetch_options: Ref<QueueFetchOptions>,
    qbit: Ref<Box<dyn QBittorrentClientTrait + Send + Sync>>,
    queue: Ref<Queue>,
}

impl QueueFetchCommand {
    /// Execute [`QueueFetchCommand`] from the CLI.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<QueueAction>> {
        if self.qbit_options.qbit_url.is_none()
            || self.qbit_options.qbit_username.is_none()
            || self.qbit_options.qbit_password.is_none()
        {
            return Err(Failure::from_action(QueueAction::FetchTorrents).with(
                "error",
                "qBittorrent URL, username, and password must be set",
            ));
        }
        let status = self.execute().await?;
        info!("{} {} items to the queue", "Added".bold(), status.added);
        trace!(
            "{} {} items already in the queue",
            "Excluded".bold(),
            status.excluded
        );
        Ok(true)
    }

    async fn execute(&self) -> Result<QueueStatus, Failure<QueueAction>> {
        let categories = &self.queue_fetch_options.qbit_queue_categories;
        let mut torrents: Vec<Torrent> = Vec::new();
        for category in categories {
            let filters = FilterOptions {
                category: Some(category.clone()),
                ..FilterOptions::default()
            };
            let response = self
                .qbit
                .get_torrents(filters)
                .await
                .map_err(Failure::wrap(QueueAction::FetchTorrents))?;
            let result = response
                .get_result("get torrents")
                .map_err(Failure::wrap(QueueAction::FetchTorrents))?;
            torrents.extend(result);
        }
        let total_from_api = torrents.len();
        torrents.retain(|t| t.amount_left == 0);
        let downloaded = torrents.len();
        trace!(
            "{} {} torrents from API ({} fully downloaded)",
            "Fetched".bold(),
            total_from_api,
            downloaded
        );
        let existing = self.queue.get_all().await?;
        let mut seen: HashSet<Hash<20>> = existing.keys().copied().collect();
        let items: BTreeMap<Hash<20>, QueueItem> = torrents
            .iter()
            .filter_map(QueueItem::from_qbit_torrent)
            .filter(|item| seen.insert(item.hash))
            .map(|item| (item.hash, item))
            .collect();
        let new_count = items.len();
        info!("{} {} new torrents", "Found".bold(), new_count);
        if new_count == 0 {
            return Ok(QueueStatus {
                success: true,
                added: 0,
                excluded: total_from_api,
            });
        }
        let added = self.queue.set_many(items, false).await?;
        Ok(QueueStatus {
            success: true,
            added,
            excluded: total_from_api - added,
        })
    }
}
