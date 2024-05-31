use crate::api::ApiError;
use crate::formats::ExistingFormat;
use crate::imdl::ImdlError;
use crate::source::SourceError::*;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SourceError {
    ApiFailure(ApiError),
    ImdlFailure(ImdlError),
    InvalidInput(String),
    AudioTagFailure(audiotags::Error),
    GroupIdNotFound,
    TorrentIdNotFound,
    GroupMisMatch(i64, i64),
    SourceDoesNotMatch(String, String),
    FileDoesNotExist(String),
    NotLossless(ExistingFormat),
    UnknownSampleRate(u32),
    TooManyChannels(u32),
    StreamInfoFailure(claxon::Error),
}

impl Display for SourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ApiFailure(error) => format!("API failure: {error}"),
            ImdlFailure(error) => format!("IMDL failure: {error}"),
            InvalidInput(input) => format!("Invalid input: {input}"),
            AudioTagFailure(error) => format!("Audio tag failure: {error}"),
            GroupIdNotFound => "Group id not found".to_owned(),
            TorrentIdNotFound => "Torrent id not found".to_owned(),
            GroupMisMatch(expected, actual) => format!("Unexpected group mismatch. Expected: {expected}, Actual: {actual}"),
            SourceDoesNotMatch(expected, actual) => format!("Source field of the torrent file does not match the tracker id. Expected: {expected}; Actual: {actual}"),
            FileDoesNotExist(path) => format!("File does not exist: {path}"),
            NotLossless(format) => format!("Source must be Flac or Flac24. Received: {format:?}"),
            UnknownSampleRate(rate) => format!("Unknown sample rate: {rate}"),
            TooManyChannels(channels) => format!("Unable to transcode more than two channels: {channels}"),
            StreamInfoFailure(error) => format!("Stream Info failure: {error}")
        };
        message.fmt(formatter)
    }
}
