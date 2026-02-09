use crate::prelude::*;

/// Actions that can fail during torrent creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TorrentCreateAction {
    #[error("build torrent")]
    BuildTorrent,
    #[error("copy torrent")]
    CopyTorrent,
    #[error("read directory")]
    ReadDirectory,
    #[error("read metadata")]
    ReadMetadata,
    #[error("read torrent")]
    ReadTorrent,
    #[error("write torrent")]
    WriteTorrent,
}
