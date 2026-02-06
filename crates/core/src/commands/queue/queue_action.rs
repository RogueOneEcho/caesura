use crate::prelude::*;

/// Actions that can fail in the queue module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum QueueAction {
    #[error("get queue item")]
    Get,
    #[error("get all queue items")]
    GetAll,
    #[error("set queue item")]
    Set,
    #[error("set many queue items")]
    SetMany,
    #[error("remove queue item")]
    Remove,
    #[error("read torrent")]
    ReadTorrent,
}
