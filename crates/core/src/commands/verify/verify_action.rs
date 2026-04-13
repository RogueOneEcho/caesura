use crate::prelude::*;

/// Actions that can fail in the verify module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum VerifyAction {
    #[error("get source from options")]
    GetSource,
    #[error("get source torrent file")]
    GetSourceTorrent,
    #[error("verify torrent hash")]
    VerifyHash,
    #[error("verify tags")]
    VerifyTags,
}
