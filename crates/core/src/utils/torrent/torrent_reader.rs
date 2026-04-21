//! Read and parse `.torrent` files.

use crate::prelude::*;
use lava_torrent::torrent::v1::Torrent as LavaTorrent;

/// Read and parse `.torrent` files using `lava_torrent`.
pub struct TorrentReader;

impl TorrentReader {
    /// Parse a `.torrent` file into a [`Torrent`].
    ///
    /// - Uses `spawn_blocking` because `lava_torrent` performs synchronous file I/O
    pub async fn execute(path: &Path) -> Result<LavaTorrent, Failure<TorrentReadAction>> {
        let path = path.to_path_buf();
        spawn_blocking(move || {
            LavaTorrent::read_from_file(&path).map_err(Failure::wrap_with_path(
                TorrentReadAction::ReadTorrent,
                &path,
            ))
        })
        .await
        .expect("torrent read task should not panic")
    }
}
