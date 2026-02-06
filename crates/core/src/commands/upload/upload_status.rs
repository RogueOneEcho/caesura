use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Not;

/// Successful result of an upload operation.
pub(crate) struct UploadSuccess {
    /// Status of each target format that was uploaded.
    pub formats: Vec<UploadFormatStatus>,
    /// Non-fatal warnings (e.g., copy failures that didn't prevent upload).
    pub warnings: Vec<rogue_logging::Error>,
}

/// Serializable status of an [`UploadCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct UploadStatus {
    /// Whether the upload operation succeeded.
    pub success: bool,
    /// Status of each target format that was uploaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<UploadFormatStatus>>,
    /// When the operation completed.
    pub completed: TimeStamp,
    /// Error or warning details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<rogue_logging::Error>>,
}

impl UploadStatus {
    /// Create a new [`UploadStatus`] from a command result.
    pub fn new(result: Result<UploadSuccess, Failure<UploadAction>>) -> Self {
        match result {
            Ok(success) => Self {
                success: true,
                formats: Some(success.formats),
                completed: TimeStamp::now(),
                errors: success
                    .warnings
                    .is_empty()
                    .not()
                    .then_some(success.warnings),
            },
            Err(failure) => Self {
                success: false,
                formats: None,
                completed: TimeStamp::now(),
                errors: Some(vec![failure.to_error()]),
            },
        }
    }
}

/// Status of a single format upload.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct UploadFormatStatus {
    /// Target format that was uploaded.
    pub format: TargetFormat,
    /// Torrent ID assigned by the tracker.
    pub id: u32,
}
