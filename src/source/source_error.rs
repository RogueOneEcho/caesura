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
    StreamInfoFailure(claxon::Error),
}