use crate::prelude::*;

/// Actions that can fail in the imdl module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum ImdlAction {
    #[error("create torrent")]
    CreateTorrent,
    #[error("read torrent")]
    ReadTorrent,
    #[error("verify torrent")]
    VerifyTorrent,
    #[error("duplicate torrent")]
    DuplicateTorrent,
    #[error("deserialize torrent")]
    Deserialize,
}
