use std::fmt::{Display, Formatter};

use clap::Args;
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::*;
use crate::commands::*;
use crate::options::*;

/// Options for [`JobRunner`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct RunnerOptions {
    /// Number of cpus to use for processing.
    ///
    /// Default: Total number of CPUs
    #[arg(long)]
    pub cpus: Option<u16>,
}

#[injectable]
impl RunnerOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for RunnerOptions {
    fn merge(&mut self, alternative: &Self) {
        if self.cpus.is_none() {
            self.cpus.clone_from(&alternative.cpus);
        }
    }

    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn apply_defaults(&mut self) {
        if self.cpus.is_none() {
            self.cpus = Some(num_cpus::get() as u16);
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        true
    }

    fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Batch { runner, .. } | Spectrogram { runner, .. } | Transcode { runner, .. }) => {
                Some(runner)
            }
            _ => None,
        }
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for RunnerOptions {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(yaml) = serde_yaml::to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
