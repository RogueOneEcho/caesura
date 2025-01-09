use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::Args;
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};
use CommandArguments::Queue;
use QueueCommandArguments::Add;

use crate::commands::*;
use crate::options::*;
/// Options for the [`QueueAddCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueueAddArgs {
    /// A path to either:
    /// - A directory of `.torrent` files
    /// - A single YAML queue file
    ///
    /// Examples: `./torrents`, `/path/to/torrents`, `./queue.yml`
    #[arg(value_name = "PATH")]
    pub queue_add_path: Option<PathBuf>,
}

#[injectable]
impl QueueAddArgs {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for QueueAddArgs {
    fn get_name() -> String {
        "Queue Arguments".to_owned()
    }

    fn merge(&mut self, alternative: &Self) {
        if self.queue_add_path.is_none() {
            self.queue_add_path.clone_from(&alternative.queue_add_path);
        }
    }

    fn apply_defaults(&mut self) {}

    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if let Some(path) = &self.queue_add_path {
            if !path.exists() {
                errors.push(DoesNotExist(
                    "Queue add path".to_owned(),
                    path.to_string_lossy().to_string(),
                ));
            }
        } else {
            errors.push(NotSet("Queue add path".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Queue {
                command: Add { args, .. },
                ..
            }) => Some(args),
            _ => None,
        }
    }

    #[allow(clippy::absolute_paths)]
    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for QueueAddArgs {
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
