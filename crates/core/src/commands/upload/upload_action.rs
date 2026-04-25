use crate::prelude::*;

/// Actions that can fail in the upload module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum UploadAction {
    #[error("get source from options")]
    GetSource,
    #[error("find torrent file")]
    FindTorrent,
    #[error("verify torrent content")]
    VerifyContent,
    #[error("upload torrent")]
    Upload,
    #[error("copy torrent")]
    CopyTorrent,
    #[error("inject torrent")]
    InjectTorrent,
    #[error("copy transcode")]
    CopyTranscode,
    #[error("get transcode command")]
    GetTranscodeCommand,
}

/// Errors that can occur during upload.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum UploadError {
    #[error("torrent file does not exist")]
    MissingTorrent,
}
