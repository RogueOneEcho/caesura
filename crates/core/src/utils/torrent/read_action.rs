use crate::prelude::*;

/// Actions that can fail during torrent reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TorrentReadAction {
    #[error("read torrent")]
    ReadTorrent,
}
