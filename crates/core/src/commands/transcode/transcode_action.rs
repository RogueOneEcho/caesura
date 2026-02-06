use crate::prelude::*;

/// Actions that can fail in the transcode module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TranscodeAction {
    #[error("get source from options")]
    GetSource,
    #[error("create output directory")]
    CreateOutputDirectory,
    #[error("hard link FLAC")]
    HardLinkFlac,
    #[error("copy FLAC")]
    CopyFlac,
    #[error("transcode")]
    Transcode,
    #[error("start decode process")]
    SpawnDecode,
    #[error("start encode process")]
    SpawnEncode,
    #[error("wait for decode")]
    WaitDecode,
    #[error("wait for encode")]
    WaitEncode,
    #[error("resample")]
    Resample,
    #[error("write tags")]
    WriteTags,
    #[error("read FLAC")]
    ReadFlac,
    #[error("get tags")]
    GetTags,
    #[error("get sample rate")]
    GetSampleRate,
    #[error("hard link additional file")]
    HardLinkAdditional,
    #[error("copy additional file")]
    CopyAdditional,
    #[error("create torrent")]
    CreateTorrent,
    #[error("resize image")]
    ResizeImage,
    #[error("execute transcode runner")]
    ExecuteRunner,
}

/// Errors that can occur during transcoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TranscodeError {
    #[error("no transcodes to perform")]
    NoTranscodes,
    #[error("unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),
}
