use crate::prelude::*;

/// Actions that can fail in the tags module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TagsAction {
    #[error("open file")]
    OpenFile,
    #[error("read tags")]
    ReadTags,
    #[error("get vorbis comments")]
    GetVorbisComments,
}

/// Errors that can occur when reading or parsing tags.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum TagsError {
    #[error("no vorbis comments found")]
    NoVorbisComments,
    #[error("no track number")]
    NoTrackNumber,
    #[error("invalid track format")]
    InvalidFormat,
}
