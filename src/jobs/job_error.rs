use std::fmt::{Display, Formatter};
use std::process::ExitStatus;
use tokio::task::JoinError;

use crate::jobs::JobError::*;
use crate::source::SourceError;

#[derive(Debug)]
pub enum JobError {
    IOFailure(std::io::Error),
    SourceFailure(SourceError),
    SpectrogramFailure {
        output_path: String,
        exit_status: ExitStatus,
        stderr: String,
        stdout: String,
    },
    JoinFailure(JoinError),
    TranscodeFailure,
}

impl Display for JobError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            IOFailure(error) => format!("IO failed: {error}"),
            SourceFailure(error) => format!("Source failure: {error}"),
            SpectrogramFailure { output_path,exit_status,stderr,stdout } =>
                format!("Failed to generate spectrogram to {output_path:?}. Exit status: {exit_status:?}. Stderr: {stderr:?}. Stdout: {stdout:?}"),
            JoinFailure(error) => format!("Join failed: {error}"),
            TranscodeFailure => "Transcode failed".to_owned()
        };
        message.fmt(formatter)
    }
}
