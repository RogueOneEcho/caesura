use crate::formats::ExistingFormat;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Format to transcode to.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TargetFormat {
    Flac,
    #[serde(rename = "320")]
    _320,
    V0,
}

impl TargetFormat {
    #[must_use]
    pub fn to_existing(self) -> ExistingFormat {
        match self {
            TargetFormat::Flac => ExistingFormat::Flac,
            TargetFormat::_320 => ExistingFormat::_320,
            TargetFormat::V0 => ExistingFormat::V0,
        }
    }

    #[must_use]
    pub fn get_file_extension(self) -> String {
        match self {
            TargetFormat::Flac => "flac".to_owned(),
            TargetFormat::_320 => "mp3".to_owned(),
            TargetFormat::V0 => "mp3".to_owned(),
        }
    }
}
