use crate::prelude::*;
use gazelle_api::GazelleSerializableError;
use reqwest::StatusCode;
use rogue_logging::Colors;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Maximum allowed path length for transcodes.
pub const MAX_PATH_LENGTH: isize = 180;
/// Minimum required bit rate in kbps.
pub const MIN_BIT_RATE_KBPS: u32 = 192;
/// Maximum allowed duration in seconds.
pub const MAX_DURATION: u32 = 12 * 60 * 60;

/// Validation issues that prevent transcoding a source.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SourceIssue {
    #[deprecated(since = "0.24.0", note = "use `Id` instead")]
    IdError {
        details: String,
    },
    Id(IdProviderError),
    GroupMismatch {
        actual: u32,
        expected: u32,
    },
    #[deprecated(since = "0.24.1", note = "use `Api` instead")]
    ApiResponse {
        action: String,
        status_code: u16,
        error: String,
    },
    #[deprecated(since = "0.24.1", note = "use `Api` instead")]
    #[allow(dead_code)]
    Provider,
    #[deprecated(since = "0.25.0", note = "use `NotFound` instead")]
    Api {
        response: GazelleSerializableError,
    },
    NotFound,
    Category {
        actual: String,
    },
    Scene,
    PossibleScene,
    LossyMaster,
    LossyWeb,
    Trumpable,
    Unconfirmed,
    Excluded {
        tags: Vec<String>,
    },
    #[deprecated(since = "0.29.0", note = "use `NoTargets` instead")]
    Existing {
        formats: BTreeSet<ExistingFormat>,
    },
    NoTargets {
        formats: BTreeSet<TargetFormat>,
    },
    NotSource {
        format: String,
        encoding: String,
    },
    MissingDirectory {
        path: PathBuf,
    },
    UnnecessaryDirectory {
        prefix: PathBuf,
    },
    NoFlacs {
        path: PathBuf,
    },
    FlacCount {
        expected: usize,
        actual: usize,
    },
    #[deprecated(
        since = "0.27.0",
        note = "split into `HashCheck`, `MissingFile`, and `OpenFile`"
    )]
    Imdl {
        details: String,
    },
    HashCheck {
        piece_index: usize,
    },
    MissingFile {
        path: PathBuf,
    },
    OpenFile {
        path: PathBuf,
        error: String,
    },
    ExcessContent,
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
    #[expect(
        deprecated,
        clippy::too_many_lines,
        reason = "match arms for many variants"
    )]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        use SourceIssue::*;
        let message = match self {
            Id(error) => error.to_string(),
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
            Provider => "Received unsuccessful API response".to_owned(),
            Api { response: issue } => issue.to_string(),
            NotFound => "Torrent not found".to_owned(),
            GroupMismatch { actual, expected } => {
                format!("Group of torrent `{actual}` did not match torrent group `{expected}`")
            }
            Category { actual } => format!("Category was not Music: {actual}"),
            Scene => "Scene releases are not supported".to_owned(),
            PossibleScene => {
                "File path and file list contain no spaces, likely a scene release".to_owned()
            }
            LossyMaster => "Lossy master releases need approval".to_owned(),
            LossyWeb => "Lossy web releases need approval".to_owned(),
            Trumpable => "Source is trumpable".to_owned(),
            Unconfirmed => "Source is missing edition information".to_owned(),
            Excluded { tags } => format!("Excluded tags: {}", join_humanized(tags)),
            Existing { formats } => {
                format!(
                    "All allowed formats have been transcoded to already: {}",
                    join_humanized(formats)
                )
            }
            NoTargets { formats } => {
                format!(
                    "All allowed formats have been transcoded to already: {}",
                    join_humanized(formats)
                )
            }
            NotSource { format, encoding } => format!("Not a suitable source: {format} {encoding}"),
            MissingDirectory { path } => {
                format!("Source directory does not exist: {}", path.display())
            }
            UnnecessaryDirectory { prefix } => {
                format!(
                    "Source content is nested within an unnecessary directory: {}",
                    prefix.display()
                )
            }
            NoFlacs { path } => format!(
                "No FLAC files found in source directory: {}",
                path.display()
            ),
            FlacCount { expected, actual } => {
                format!("Expected {expected} FLACs, found {actual}")
            }
            Imdl { details } => format!("Files do not match hash:\n{details}"),
            HashCheck { piece_index } => {
                format!("Piece {piece_index} hash mismatch")
            }
            MissingFile { path } => {
                format!("Expected file not found: {}", path.display())
            }
            OpenFile { path, error } => {
                format!("Failed to open {}: {error}", path.display())
            }
            ExcessContent => "Content is larger than expected by torrent".to_owned(),
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
            Error { domain, details } => format!("A {domain} error occurred:\n{details}"),
            Other(details) => details.clone(),
        };
        write!(formatter, "{message}")
    }
}

impl Error for SourceIssue {}
