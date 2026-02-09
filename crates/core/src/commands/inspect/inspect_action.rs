use crate::prelude::*;

/// Actions that can fail when inspecting audio files.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum InspectAction {
    #[error("read directory")]
    ReadDir,
    #[error("open audio file")]
    OpenFile,
    #[error("read FLAC file")]
    ReadFlacFile,
    #[error("read MPEG file")]
    ReadMpegFile,
}

/// Errors returned when inspecting audio files.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum InspectError {
    #[error("unsupported file extension")]
    UnsupportedExtension,
}
