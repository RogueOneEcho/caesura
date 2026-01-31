use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Error as IoError, ErrorKind};

use crate::utils::ProcessError;

/// Errors that can occur during sample generation.
#[derive(Debug)]
pub enum SampleError {
    /// Failed to create a directory.
    CreateDirectory(IoError),
    /// Failed to remove a temporary file.
    RemoveFile(IoError),

    /// SOX command failed.
    Sox(ProcessError),

    /// metaflac failed to set tags.
    MetaflacTags(ProcessError),
    /// metaflac failed to import picture.
    MetaflacPicture(ProcessError),

    /// Failed to save image.
    ImageSave(image::ImageError),

    /// Failed to create torrent.
    TorrentCreate(rogue_logging::Error),

    /// Transcode operation failed.
    Transcode(String),
}

impl Display for SampleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::CreateDirectory(e) => write!(f, "Failed to create directory\n{e}"),
            Self::RemoveFile(e) => write!(f, "Failed to remove file\n{e}"),
            Self::Sox(e) => fmt_command_error(f, e, "sox", "Failed to generate FLAC"),
            Self::MetaflacTags(e) => fmt_command_error(f, e, "metaflac", "Failed to set tags"),
            Self::MetaflacPicture(e) => {
                fmt_command_error(f, e, "metaflac", "Failed to import picture")
            }
            Self::ImageSave(e) => write!(f, "Failed to save image\n{e}"),
            Self::TorrentCreate(e) => write!(f, "Failed to create torrent\n{e}"),
            Self::Transcode(e) => write!(f, "Transcode failed\n{e}"),
        }
    }
}

impl Error for SampleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CreateDirectory(e) | Self::RemoveFile(e) => Some(e),
            Self::Sox(e) | Self::MetaflacTags(e) | Self::MetaflacPicture(e) => e.source(),
            Self::ImageSave(e) => Some(e),
            Self::TorrentCreate(e) => Some(e),
            Self::Transcode(_) => None,
        }
    }
}

impl From<image::ImageError> for SampleError {
    fn from(err: image::ImageError) -> Self {
        Self::ImageSave(err)
    }
}

impl From<rogue_logging::Error> for SampleError {
    fn from(err: rogue_logging::Error) -> Self {
        Self::TorrentCreate(err)
    }
}

/// Format a command error with a custom message, handling "not found" specially.
fn fmt_command_error(
    f: &mut Formatter<'_>,
    error: &ProcessError,
    command: &str,
    action: &str,
) -> FmtResult {
    match error {
        ProcessError::Spawn(e) if e.kind() == ErrorKind::NotFound => {
            write!(f, "`{command}` not found\nIs it installed and in PATH?")
        }
        ProcessError::Spawn(e) | ProcessError::Wait(e) => {
            write!(f, "Failed to run `{command}`\n{e}")
        }
        ProcessError::Failed(output) => write!(f, "{action} with `{command}`\n{output}"),
    }
}
