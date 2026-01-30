use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{Batch, Queue};
use crate::commands::QueueCommandArguments::*;
use crate::commands::*;
use crate::options::*;
use caesura_macros::Options;

/// Options for batch processing
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Queue))]
#[options(from_args_fn = "Self::partial_from_args")]
#[allow(clippy::struct_excessive_bools)]
pub struct BatchOptions {
    /// Should the spectrogram command be executed?
    ///
    /// Default: `false`
    #[arg(long)]
    pub spectrogram: bool,

    /// Should the transcode command be executed?
    ///
    /// Default: `false`
    #[arg(long)]
    pub transcode: bool,

    /// Should failed transcodes be retried?
    ///
    /// Default: `false`
    #[arg(long)]
    pub retry_transcode: bool,

    /// Should the upload command be executed?
    ///
    /// Default: `false`
    #[arg(long)]
    pub upload: bool,

    /// Limit the number of torrents to batch process.
    ///
    /// If `no_limit` is set, this option is ignored.
    ///
    /// Default: `3`
    #[arg(long)]
    #[options(default = 3)]
    pub limit: usize,

    /// Should the `limit` option be ignored?
    ///
    /// Default: `false`
    #[arg(long)]
    pub no_limit: bool,

    /// Wait for a duration before uploading the torrent.
    ///
    /// The duration is a string that can be parsed such as `500ms`, `5m`, `1h30m15s`.
    ///
    /// Default: `null`
    #[arg(long)]
    pub wait_before_upload: Option<String>,
}

impl BatchOptions {
    #[must_use]
    pub fn get_wait_before_upload(&self) -> Option<Duration> {
        let wait_before_upload = self.wait_before_upload.clone()?;
        humantime::parse_duration(wait_before_upload.as_str()).ok()
    }

    #[must_use]
    pub fn get_limit(&self) -> Option<usize> {
        if self.no_limit {
            None
        } else {
            Some(self.limit)
        }
    }

    /// Custom `from_args` implementation for complex Queue subcommand matching
    #[allow(clippy::manual_let_else)]
    #[must_use]
    pub fn partial_from_args() -> Option<BatchOptionsPartial> {
        match ArgumentsParser::get() {
            Some(
                Batch { batch, .. }
                | Queue {
                    command: List { batch, .. },
                },
            ) => Some(batch),
            _ => None,
        }
    }
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            spectrogram: false,
            transcode: false,
            retry_transcode: false,
            upload: false,
            limit: 3,
            no_limit: false,
            wait_before_upload: None,
        }
    }
}

impl BatchOptions {
    /// Validate the partial options.
    pub fn validate_partial(partial: &BatchOptionsPartial, errors: &mut Vec<OptionRule>) {
        if let Some(wait_before_upload) = &partial.wait_before_upload
            && humantime::parse_duration(wait_before_upload.as_str()).is_err()
        {
            errors.push(OptionRule::DurationInvalid(
                "Wait Before Upload".to_owned(),
                wait_before_upload.clone(),
            ));
        }
        if partial.upload == Some(true) && partial.transcode != Some(true) {
            errors.push(OptionRule::Dependent(
                "Upload".to_owned(),
                "Transcode".to_owned(),
            ));
        }
    }
}
