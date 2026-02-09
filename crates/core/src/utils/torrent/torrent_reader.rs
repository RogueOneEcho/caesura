//! Read and parse `.torrent` files.

use lava_torrent::torrent::v1::Torrent;
use rogue_logging::Failure;
use std::path::Path;
use tokio::task::spawn_blocking;

use super::TorrentReadAction;

/// Read and parse `.torrent` files using `lava_torrent`.
pub struct TorrentReader;

impl TorrentReader {
    /// Parse a `.torrent` file into a [`Torrent`].
    ///
    /// - Uses `spawn_blocking` because `lava_torrent` performs synchronous file I/O
    pub async fn execute(path: &Path) -> Result<Torrent, Failure<TorrentReadAction>> {
        let path = path.to_path_buf();
        spawn_blocking(move || {
            Torrent::read_from_file(&path).map_err(Failure::wrap_with_path(
                TorrentReadAction::ReadTorrent,
                &path,
            ))
        })
        .await
        .expect("torrent read task should not panic")
    }
}
