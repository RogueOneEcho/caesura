use crate::prelude::*;

/// Information needed to resample a FLAC.
pub(crate) struct Resample {
    /// Path to the input file
    pub input: PathBuf,
    /// Path to the output file
    pub output: PathBuf,
    /// Resample rate
    pub resample_rate: u32,
    /// Use repeatable mode for `SoX` (deterministic dithering)
    pub repeatable: bool,
}

impl Resample {
    /// Create a new resample command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        let mut args = Vec::new();
        if self.repeatable {
            args.push("-R".to_owned());
        }
        args.extend([
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
        ]);
        CommandInfo {
            program: SOX.to_owned(),
            args,
        }
    }
}
