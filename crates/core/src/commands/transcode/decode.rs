use crate::prelude::*;

/// Information needed to decode a FLAC file to raw audio.
pub(crate) struct Decode {
    /// Path to the input file
    pub input: PathBuf,
    /// Optional resample rate
    pub resample_rate: Option<u32>,
    /// Use repeatable mode for `SoX` (deterministic dithering)
    pub repeatable: bool,
    /// Factory for creating sox commands
    pub sox: Ref<SoxFactory>,
}

impl Decode {
    /// Get the [`CommandInfo`] for the decode command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        match self.resample_rate {
            Some(rate) => self.decode_with_resample(rate),
            None => decode_without_resample(self.input),
        }
    }

    fn decode_with_resample(self, resample_rate: u32) -> CommandInfo {
        let mut info = self.sox.create();
        if self.repeatable {
            info.args.push("-R".to_owned());
        }
        info.args.extend([
            self.input.to_string_lossy().to_string(),
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
        ]);
        info
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
