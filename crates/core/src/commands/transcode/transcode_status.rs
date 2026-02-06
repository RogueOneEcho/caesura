use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Successful result of a transcode operation.
pub(crate) struct TranscodeSuccess {
    /// Status of each target format that was transcoded.
    pub formats: Vec<TranscodeFormatStatus>,
}

/// Serializable status of a [`TranscodeCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct TranscodeStatus {
    /// Whether the transcode operation succeeded.
    pub success: bool,
    /// Status of each target format that was transcoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<TranscodeFormatStatus>>,
    /// When the operation completed.
    pub completed: TimeStamp,
    /// Error details if the operation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<rogue_logging::Error>,
}

impl TranscodeStatus {
    /// Create a new [`TranscodeStatus`] from a command result.
    pub fn new(result: Result<TranscodeSuccess, Failure<TranscodeAction>>) -> Self {
        match result {
            Ok(success) => Self {
                success: true,
                formats: Some(success.formats),
                completed: TimeStamp::now(),
                error: None,
            },
            Err(failure) => Self {
                success: false,
                formats: None,
                completed: TimeStamp::now(),
                error: Some(failure.to_error()),
            },
        }
    }
}

/// Status of a single format transcode.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct TranscodeFormatStatus {
    /// Target format that was transcoded.
    pub format: TargetFormat,
    /// Output directory containing the transcoded files.
    pub path: PathBuf,
}
