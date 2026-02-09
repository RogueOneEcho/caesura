use crate::prelude::*;

/// Actions that can fail during torrent verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TorrentVerifyAction {
    #[error("read torrent")]
    ReadTorrent,
    #[error("hash content")]
    HashContent,
}
