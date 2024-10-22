use crate::formats::ExistingFormat;
use crate::source::SourceIssue::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub const MAX_PATH_LENGTH: isize = 180;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SourceIssue {
    IdError {
        details: String,
    },
    GroupMismatch {
        actual: i64,
        expected: i64,
    },
    ApiResponse {
        action: String,
        status_code: u16,
        error: String,
    },
    Category {
        actual: String,
    },
    Scene,
    LossyMaster,
    LossyWeb,
    Trumpable,
    Existing {
        formats: BTreeSet<ExistingFormat>,
    },
    MissingDirectory {
        path: PathBuf,
    },
    NoFlacs {
        path: PathBuf,
    },
    Imdl {
        details: String,
    },
    Length {
        path: PathBuf,
        excess: usize,
    },
    MissingTags {
        path: PathBuf,
        tags: Vec<String>,
    },
    FlacError {
        path: PathBuf,
        error: String,
    },
    SampleRate {
        path: PathBuf,
        rate: u32,
    },
    Channels {
        path: PathBuf,
        count: u32,
    },
    Error {
        domain: String,
        details: String,
    },
}

impl Display for SourceIssue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            IdError { details } => format!("Invalid source id: {details}"),
            ApiResponse {
                action,
                status_code,
                error,
            } => {
                let status = StatusCode::from_u16(*status_code)
                    .expect("Status code is valid")
                    .canonical_reason()
                    .unwrap_or("Unknown");
                format!("API responded {status} to {action}: {error}")
            }
            GroupMismatch { actual, expected } => {
                format!("Group of torrent `{actual}` did not match torrent group `{expected}`")
            }
            Category { actual } => format!("Category was not Music: {actual}"),
            Scene => "Scene releases are not supported".to_owned(),
            LossyMaster => "Lossy master releases need approval".to_owned(),
            LossyWeb => "Lossy web releases need approval".to_owned(),
            Trumpable => "Source is trumpable".to_owned(),
            Existing { formats } => {
                format!("All allowed formats have been transcoded to already: {formats:?}",)
            }
            MissingDirectory { path } => format!("Source directory not found: {path:?}"),
            NoFlacs { path } => format!("No FLAC files found in source directory: {path:?}"),
            Imdl { details } => format!("Files do not match hash:\n{details}"),
            Length { path, excess } => {
                format!("Path is {excess} characters longer than allowed: {path:?}")
            }
            MissingTags { path, tags } => format!("Missing tags: {tags:?}: {path:?}"),
            SampleRate { path, rate } => format!("Unsupported sample rate: {rate}: {path:?}"),
            Channels { path, count } => {
                format!("Too many channels: {count}: {path:?}")
            }
            FlacError { path, error } => format!("FLAC stream error: {error}: {path:?}"),
            Error { domain, details } => format!("A {domain} error occured:\n{details}"),
        };
        message.fmt(formatter)
    }
}