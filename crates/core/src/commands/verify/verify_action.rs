use crate::prelude::*;

/// Actions that can fail in the verify module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum VerifyAction {
    #[error("get source from options")]
    GetSource,
    #[error("create torrent cache directory")]
    CreateTorrentDirectory,
    #[error("create source torrent file")]
    CreateTorrentFile,
    #[error("download source torrent")]
    DownloadTorrent,
    #[error("write source torrent file")]
    WriteTorrentFile,
    #[error("flush source torrent file")]
    FlushTorrentFile,
    #[error("rename source torrent file")]
    RenameTorrentFile,
    #[error("verify torrent hash")]
    VerifyHash,
    #[error("verify tags")]
    VerifyTags,
}
