use crate::prelude::*;
use TargetFormat::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Format to transcode to.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TargetFormat {
    Flac = 1,
    #[serde(rename = "320")]
    _320 = 2,
    V0 = 3,
}

impl TargetFormat {
    /// Display name of the format.
    #[must_use]
    pub fn get_name(&self) -> &str {
        match self {
            Flac => "FLAC",
            _320 => "320",
            V0 => "V0",
        }
    }

    /// Convert to the equivalent [`ExistingFormat`].
    #[must_use]
    pub fn to_existing(self) -> ExistingFormat {
        match self {
            Flac => ExistingFormat::Flac,
            _320 => ExistingFormat::_320,
            V0 => ExistingFormat::V0,
        }
    }

    /// File extension for this format.
    #[must_use]
    pub fn get_file_extension(self) -> String {
        match self {
            Flac => "flac".to_owned(),
            _320 | V0 => "mp3".to_owned(),
        }
    }

    /// Bitrate description for the API.
    #[must_use]
    pub fn get_bitrate(&self) -> &str {
        match self {
            Flac => "Lossless",
            _320 => "320",
            V0 => "V0 (VBR)",
        }
    }
}

impl Display for TargetFormat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.get_name())
    }
}

impl PartialOrd for TargetFormat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TargetFormat {
    #[allow(clippy::as_conversions)]
    fn cmp(&self, other: &Self) -> Ordering {
        let left = *self as isize;
        let right = *other as isize;
        left.cmp(&right)
    }
}
