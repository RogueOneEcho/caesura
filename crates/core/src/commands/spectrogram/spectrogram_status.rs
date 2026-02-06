use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Successful result of a spectrogram operation.
pub(crate) struct SpectrogramSuccess {
    /// Output directory containing generated spectrograms.
    pub path: PathBuf,
    /// Number of spectrograms generated.
    pub count: usize,
}

/// Serializable status of a [`SpectrogramCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct SpectrogramStatus {
    /// Whether the spectrogram operation succeeded.
    pub success: bool,
    /// Output directory containing generated spectrograms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Number of spectrograms generated.
    pub count: usize,
    /// When the operation completed.
    pub completed: TimeStamp,
    /// Error details if the operation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<rogue_logging::Error>,
}

impl SpectrogramStatus {
    /// Create a new [`SpectrogramStatus`] from a command result.
    pub fn new(result: Result<SpectrogramSuccess, Failure<SpectrogramAction>>) -> Self {
        match result {
            Ok(success) => Self {
                success: true,
                path: Some(success.path),
                count: success.count,
                completed: TimeStamp::now(),
                error: None,
            },
            Err(failure) => Self {
                success: false,
                path: None,
                count: 0,
                completed: TimeStamp::now(),
                error: Some(failure.to_error()),
            },
        }
    }
}
