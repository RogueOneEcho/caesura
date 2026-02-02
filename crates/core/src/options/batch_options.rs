use crate::commands::CommandArguments::{self, *};
use crate::commands::QueueCommandArguments;
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Options for batch processing
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct BatchOptions {
    /// Should the spectrogram command be executed?
    #[arg(long)]
    pub spectrogram: bool,

    /// Should the transcode command be executed?
    #[arg(long)]
    pub transcode: bool,

    /// Should failed transcodes be retried?
    #[arg(long)]
    pub retry_transcode: bool,

    /// Should the upload command be executed?
    #[arg(long)]
    pub upload: bool,

    /// Limit the number of torrents to batch process.
    ///
    /// If `no_limit` is set, this option is ignored.
    #[arg(long)]
    #[options(default = 3)]
    pub limit: usize,

    /// Should the `limit` option be ignored?
    #[arg(long)]
    pub no_limit: bool,

    /// Wait for a duration before uploading the torrent.
    ///
    /// The duration is a string that can be parsed such as `500ms`, `5m`, `1h30m15s`.
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
}

impl OptionsContract for BatchOptions {
    type Partial = BatchOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(wait_before_upload) = &self.wait_before_upload
            && humantime::parse_duration(wait_before_upload.as_str()).is_err()
        {
            errors.push(OptionRule::DurationInvalid(
                "Wait Before Upload".to_owned(),
                wait_before_upload.clone(),
            ));
        }
        if self.upload && !self.transcode {
            errors.push(OptionRule::Dependent(
                "Upload".to_owned(),
                "Transcode".to_owned(),
            ));
        }
    }
}

impl FromArgs for BatchOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(
                Batch { batch, .. }
                | CommandArguments::Queue {
                    command: QueueCommandArguments::List { batch, .. },
                },
            ) => Some(batch.clone()),
            _ => None,
        }
    }
}
