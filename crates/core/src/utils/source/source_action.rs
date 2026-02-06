use crate::prelude::*;

/// Actions that can fail in the source module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum SourceAction {
    #[error("get torrent")]
    GetTorrent,
    #[error("get torrent group")]
    GetTorrentGroup,
}
