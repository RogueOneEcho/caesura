use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Result of a [`SpectrogramCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct SpectrogramStatus {
    /// Did the spectrogram command succeed?
    pub success: bool,
    /// Path to the spectrogram directory
    pub path: Option<PathBuf>,
    /// Number of spectrograms created
    pub count: usize,
    /// Time the spectrogram completed
    pub completed: TimeStamp,
    /// Error message if the spectrogram failed
    pub error: Option<Error>,
}
