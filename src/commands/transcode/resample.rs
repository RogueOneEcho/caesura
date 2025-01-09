use crate::commands::*;
use crate::dependencies::*;

use std::path::PathBuf;

/// Information needed to resample a FLAC.
pub(crate) struct Resample {
    /// Path to the input file
    pub input: PathBuf,
    /// Path to the output file
    pub output: PathBuf,
    /// Resample rate
    #[allow(clippy::struct_field_names)]
    pub resample_rate: u32,
}

impl Resample {
    /// Create a new resample command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        CommandInfo {
            program: SOX.to_owned(),
            args: vec![
                self.input.to_string_lossy().to_string(),
                "-G".to_owned(),
                "-b".to_owned(),
                "16".to_owned(),
                self.output.to_string_lossy().to_string(),
                "rate".to_owned(),
                "-v".to_owned(),
                "-L".to_owned(),
                self.resample_rate.to_string(),
                "dither".to_owned(),
            ],
        }
    }
}
