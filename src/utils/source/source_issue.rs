use crate::utils::*;
use SourceIssue::*;

use reqwest::StatusCode;
use rogue_logging::Colors;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub const MAX_PATH_LENGTH: isize = 180;
pub const MIN_BIT_RATE_KBPS: u32 = 192;
pub const MAX_DURATION: u32 = 12 * 60 * 60;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SourceIssue {
    IdError {
        details: String,
    },
    GroupMismatch {
        actual: u32,
        expected: u32,
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
    Unconfirmed,
    Excluded {
        tags: Vec<String>,
    },
    Existing {
        formats: BTreeSet<ExistingFormat>,
    },
    NotSource {
        format: String,
        encoding: String,
    },
    MissingDirectory {
        path: PathBuf,
    },
    NoFlacs {
        path: PathBuf,
    },
    FlacCount {
        expected: usize,
        actual: usize,
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
    BitRate {
        path: PathBuf,
        rate: u32,
    },
    Duration {
        path: PathBuf,
        seconds: u32,
    },
    Channels {
        path: PathBuf,
        count: u32,
    },
    Error {
        domain: String,
        details: String,
    },
    Other(String),
}

impl Display for SourceIssue {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            IdError { details } => format!("Invalid source id: {details}"),
            ApiResponse {
                action,
                status_code,
                error,
            } => {
                let status = StatusCode::from_u16(*status_code)
                    .ok()
                    .and_then(|code| code.canonical_reason())
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
            Unconfirmed => "Unconfirmed Release need to be confirmed".to_owned(),
            Excluded { tags } => format!("Excluded tags: {}", join_humanized(tags)),
            Existing { formats } => {
                format!(
                    "All allowed formats have been transcoded to already: {}",
                    join_humanized(formats)
                )
            }
            NotSource { format, encoding } => format!("Not a suitable source: {format} {encoding}"),
            MissingDirectory { path } => {
                format!("Source directory does not exist: {}", path.display())
            }
            NoFlacs { path } => format!(
                "No FLAC files found in source directory: {}",
                path.display()
            ),
            FlacCount { expected, actual } => {
                format!("Expected {expected} FLACs, found {actual}")
            }
            Imdl { details } => format!("Files do not match hash:\n{details}"),
            Length { path, excess } => {
                format!(
                    "Path is {excess} characters longer than allowed:\n{}",
                    path.display().to_string().gray()
                )
            }
            Duration { path, seconds } => {
                let minutes = seconds / 60;
                format!(
                    "Duration is excessive: {minutes} minutes: {}",
                    path.display()
                )
            }
            MissingTags { path, tags } => {
                format!("Missing tags: {}: {}", join_humanized(tags), path.display())
            }
            SampleRate { path, rate } => {
                format!("Unsupported sample rate: {rate}: {}", path.display())
            }
            BitRate { path, rate } => {
                format!(
                    "Bit rate was less than {MIN_BIT_RATE_KBPS} kbps: {rate}: {}",
                    path.display()
                )
            }
            Channels { path, count } => {
                format!("Too many channels: {count}: {}", path.display())
            }
            FlacError { path, error } => format!("FLAC stream error: {error}: {}", path.display()),
            Error { domain, details } => format!("A {domain} error occured:\n{details}"),
            Other(details) => details.clone(),
        };
        message.fmt(formatter)
    }
}
