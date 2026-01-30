use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Spectrogram image size.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    /// Full track spectrogram.
    Full,
    /// Zoomed 2-second sample at 1:00.
    Zoom,
}
