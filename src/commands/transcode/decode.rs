use crate::commands::*;
use crate::dependencies::*;

use std::path::PathBuf;

pub(crate) struct Decode {
    /// Path to the input file
    pub input: PathBuf,
    /// Optional resample rate
    pub resample_rate: Option<u32>,
}

impl Decode {
    /// Get the [`CommandInfo`] for the decode command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        match self.resample_rate {
            Some(rate) => decode_with_resample(self.input, rate),
            None => decode_without_resample(self.input),
        }
    }
}

fn decode_with_resample(input: PathBuf, resample_rate: u32) -> CommandInfo {
    CommandInfo {
        program: SOX.to_owned(),
        args: vec![
            input.to_string_lossy().to_string(),
            "-G".to_owned(),
            "-b".to_owned(),
            "16".to_owned(),
            "-t".to_owned(),
            "wav".to_owned(),
            "-".to_owned(),
            "rate".to_owned(),
            "-v".to_owned(),
            "-L".to_owned(),
            resample_rate.to_string(),
            "dither".to_owned(),
        ],
    }
}

fn decode_without_resample(input: PathBuf) -> CommandInfo {
    CommandInfo {
        program: FLAC.to_owned(),
        args: vec![
            "-dcs".to_owned(),
            "--".to_owned(),
            input.to_string_lossy().to_string(),
        ],
    }
}
