use crate::prelude::*;

/// Actions that can fail in the fs module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum FsAction {
    #[error("create directory")]
    CreateDirectory,
    #[error("hard link file")]
    HardLink,
    #[error("copy file")]
    CopyFile,
    #[error("copy directory")]
    CopyDirectory,
    #[error("read directory")]
    ReadDirectory,
    #[error("open file")]
    OpenFile,
    #[error("read metadata")]
    ReadMetadata,
}
