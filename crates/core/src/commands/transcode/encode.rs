use crate::prelude::*;
use crate::utils::TargetFormat::*;

/// Information required to create an encode command [`Command`].
pub(crate) struct Encode {
    /// Path to the output file.
    pub output: PathBuf,
    /// Target format to encode to.
    pub format: TargetFormat,
}

impl Encode {
    /// Get the [`CommandInfo`] for the encode command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        match self.format {
            Flac => encode_flac(self.output),
            _320 => encode_mp3_320(self.output),
            V0 => encode_mp3_v0(self.output),
        }
    }
}

fn encode_mp3_v0(output_path: PathBuf) -> CommandInfo {
    CommandInfo {
        program: LAME.to_owned(),
        args: vec![
            "-S".to_owned(),
            "-V".to_owned(),
            "0".to_owned(),
            "--vbr-new".to_owned(),
            "--ignore-tag-errors".to_owned(),
            "-".to_owned(),
            output_path.to_string_lossy().to_string(),
        ],
    }
}

fn encode_mp3_320(output_path: PathBuf) -> CommandInfo {
    CommandInfo {
        program: LAME.to_owned(),
        args: vec![
            "-S".to_owned(),
            "-h".to_owned(),
            "-b".to_owned(),
            "320".to_owned(),
            "--ignore-tag-errors".to_owned(),
            "-".to_owned(),
            output_path.to_string_lossy().to_string(),
        ],
    }
}

fn encode_flac(output_path: PathBuf) -> CommandInfo {
    CommandInfo {
        program: FLAC.to_owned(),
        args: vec![
            "--best".to_owned(),
            "-o".to_owned(),
            output_path.to_string_lossy().to_string(),
            "-".to_owned(),
        ],
    }
}
