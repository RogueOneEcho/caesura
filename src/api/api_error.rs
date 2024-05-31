use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::api::ApiError::*;

#[derive(Debug)]
pub enum ApiError {
    ClientFailure(reqwest::Error),
    RequestFailure(String, reqwest::Error),
    DeserializationFailure(String, reqwest::Error),
    InvalidInput,
    GroupIdNotFound,
    TorrentIdNotFound,
    TorrentNotFound,
    GroupNotFound,
    GroupMisMatch,
    NotRedactedTorrent,
    FileDoesNotExist(PathBuf),
    EmptyResponse,
}

impl Display for ApiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ClientFailure(error) => format!("Client failed: {error}"),
            RequestFailure(url, error) => format!("API request ({url}) failed: {error}"),
            DeserializationFailure(url, error) => {
                format!("API request ({url}) deserialization failed: {error}")
            }
            InvalidInput => "Invalid input".to_owned(),
            GroupIdNotFound => "Group id not found".to_owned(),
            TorrentIdNotFound => "Torrent id not found".to_owned(),
            TorrentNotFound => "Torrent not found".to_owned(),
            GroupNotFound => "Group not found".to_owned(),
            GroupMisMatch => "Unexpected group mismatch".to_owned(),
            NotRedactedTorrent => "Torrent file is not red".to_owned(),
            FileDoesNotExist(path) => format!("File does not exist: {path:?}"),
            EmptyResponse => {
                "API response status was success but response was unexpectedly empty.".to_owned()
            }
        };
        message.fmt(formatter)
    }
}
