use claxon::metadata::StreamInfo;
use tokio::process::Command;

use crate::dependencies::{FLAC, LAME, SOX};
use crate::errors::AppError;
use crate::formats::target_format::TargetFormat;
use crate::formats::target_format::TargetFormat::*;
use crate::fs::FlacFile;
use crate::transcode::{get_resample_rate_or_err, is_resample_required};

pub struct CommandFactory {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandFactory {
    pub fn create(self) -> Command {
        let mut cmd = Command::new(self.program);
        cmd.args(self.args);
        cmd
    }

    pub fn new_flac_resample(
        flac: &FlacFile,
        info: &StreamInfo,
        output_path: String,
    ) -> Result<CommandFactory, AppError> {
        let resample_rate = get_resample_rate_or_err(info)?;
        let command = CommandFactory {
            program: SOX.to_owned(),
            args: vec![
                flac.get_path_string(),
                "-G".to_owned(),
                "-b".to_owned(),
                "16".to_owned(),
                output_path,
                "rate".to_owned(),
                "-v".to_owned(),
                "-L".to_owned(),
                resample_rate.to_string(),
                "dither".to_owned(),
            ],
        };
        Ok(command)
    }

    pub fn new_decode(flac: &FlacFile, info: &StreamInfo) -> Result<CommandFactory, AppError> {
        let command = if is_resample_required(info) {
            let resample_rate = get_resample_rate_or_err(info)?;
            decode_with_resample(flac, resample_rate)
        } else {
            decode_without_resample(flac)
        };
        Ok(command)
    }

    pub fn new_encode(format: TargetFormat, output_path: String) -> CommandFactory {
        match format {
            Flac => encode_flac(output_path),
            _320 => encode_mp3_320(output_path),
            V0 => encode_mp3_v0(output_path),
        }
    }
}

fn decode_with_resample(flac: &FlacFile, resample_rate: u32) -> CommandFactory {
    CommandFactory {
        program: SOX.to_owned(),
        args: vec![
            flac.get_path_string(),
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

fn decode_without_resample(flac: &FlacFile) -> CommandFactory {
    CommandFactory {
        program: FLAC.to_owned(),
        args: vec!["-dcs".to_owned(), "--".to_owned(), flac.get_path_string()],
    }
}

fn encode_mp3_v0(output_path: String) -> CommandFactory {
    CommandFactory {
        program: LAME.to_owned(),
        args: vec![
            "-S".to_owned(),
            "-V".to_owned(),
            "0".to_owned(),
            "--vbr-new".to_owned(),
            "--ignore-tag-errors".to_owned(),
            "-".to_owned(),
            output_path,
        ],
    }
}

fn encode_mp3_320(output_path: String) -> CommandFactory {
    CommandFactory {
        program: LAME.to_owned(),
        args: vec![
            "-S".to_owned(),
            "-h".to_owned(),
            "-b".to_owned(),
            "320".to_owned(),
            "--ignore-tag-errors".to_owned(),
            "-".to_owned(),
            output_path,
        ],
    }
}

fn encode_flac(output_path: String) -> CommandFactory {
    CommandFactory {
        program: FLAC.to_owned(),
        args: vec![
            "--best".to_owned(),
            "-o".to_owned(),
            output_path,
            "-".to_owned(),
        ],
    }
}
