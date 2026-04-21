use crate::prelude::*;
use reqwest::StatusCode;

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
    #[expect(deprecated, reason = "match arms for deprecated variants")]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        use SourceIssue::*;
        let message = match self {
            Id(error) => error.to_string(),
            IdError { details } => format!("Invalid id: {details}"),
            ApiResponse {
                action,
                status_code,
                error,
            } => {
                format!(
                    "API responded {} to {action}: {error}",
                    get_status(status_code)
                )
            }
            Provider => "Received unsuccessful API response".to_owned(),
            Api { response: issue } => issue.to_string(),
            NotFound => "Torrent not found".to_owned(),
            GroupMismatch { actual, expected } => {
                format!("Group of torrent `{actual}` did not match torrent group `{expected}`")
            }
            Category { actual } => format!("Unsupported category: {actual}"),
            Scene => "Unsupported scene".to_owned(),
            PossibleScene => "Possible scene".to_owned(),
            LossyMaster => "Lossy master needs approval".to_owned(),
            LossyWeb => "Lossy web needs approval".to_owned(),
            Trumpable => "Trumpable".to_owned(),
            Unconfirmed => "Missing edition information".to_owned(),
            Excluded { tags } => format!("Excluded tags: {}", join_humanized(tags)),
            Existing { formats } => {
                format!("Already transcoded: {}", join_humanized(formats))
            }
            NoTargets { formats } => {
                format!("Already transcoded: {}", join_humanized(formats))
            }
            NotSource { format, encoding } => format!("Unsupported: {format} {encoding}"),
            MissingDirectory { path } => {
                format!("Directory not found: {}", format_path(path))
            }
            UnnecessaryDirectory { prefix } => {
                format!("Unnecessary nested directory: {}", format_path(prefix))
            }
            NoFlacs { path } => format!("No FLAC files in directory: {}", format_path(path)),
            FlacCount { expected, actual } => {
                format!("Expected {expected} FLACs, found {actual}")
            }
            Imdl { details } => format!("Files do not match hash:\n{details}"),
            HashCheck { piece_index } => {
                format!("Incorrect hash for piece {piece_index}")
            }
            MissingFile { path } => {
                format!("Missing file: {}", format_path(path))
            }
            OpenFile { path, error } => {
                format!("Failed to open file: {error}: {}", format_path(path))
            }
            ExcessContent => "Content exceeds torrent size".to_owned(),
            Length { path, excess } => {
                format!(
                    "Path is {excess} characters too long:\n{}",
                    format_path(path)
                )
            }
            Duration { path, seconds } => {
                format!(
                    "Excessive duration: {} minutes: {}",
                    seconds / 60,
                    format_path(path)
                )
            }
            MissingTags { path, tags } => {
                format!(
                    "Missing tags: {}: {}",
                    join_humanized(tags),
                    format_path(path)
                )
            }
            SampleRate { path, rate } => {
                format!("Unsupported sample rate: {rate}: {}", format_path(path))
            }
            BitRate { path, rate } => {
                format!("Bit rate too low: {rate} kbps: {}", format_path(path))
            }
            Channels { path, count } => {
                format!("Too many channels: {count}: {}", format_path(path))
            }
            FlacError { path, error } => {
                format!("FLAC stream error: {error}: {}", format_path(path))
            }
            Error { domain, details } => format!("A {domain} error occurred:\n{details}"),
            Other(details) => details.clone(),
        };
        write!(formatter, "{message}")
    }
}

impl Error for SourceIssue {}

/// Format a path as a gray string for display.
fn format_path(path: &Path) -> String {
    path.display().to_string().gray().to_string()
}

fn get_status(status_code: &u16) -> &str {
    StatusCode::from_u16(*status_code)
        .ok()
        .and_then(|code| code.canonical_reason())
        .unwrap_or("Unknown")
}
