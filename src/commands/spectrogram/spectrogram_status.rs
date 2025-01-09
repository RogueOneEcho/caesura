use crate::commands::*;
use rogue_logging::Error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
