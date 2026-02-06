//! Action types for sample generation errors.

use crate::prelude::*;

/// Action that failed during sample generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum SampleAction {
    /// Failed to create a directory.
    #[error("create directory")]
    CreateDirectory,
    /// Failed to remove a temporary file.
    #[error("remove file")]
    RemoveFile,
    /// Failed to generate a FLAC file with SOX.
    #[error("generate FLAC")]
    GenerateFlac,
    /// Failed to set metadata tags with metaflac.
    #[error("set tags")]
    SetTags,
    /// Failed to import a picture with metaflac.
    #[error("import picture")]
    ImportPicture,
    /// Failed to save an image.
    #[error("save image")]
    SaveImage,
    /// Failed to create a torrent.
    #[error("create torrent")]
    CreateTorrent,
    /// Transcode operation failed.
    #[error("transcode")]
    Transcode,
}
