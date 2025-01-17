use std::fmt::{Display, Formatter};

use clap::Args;
use di::{injectable, Ref};
use flat_db::Hash;
use serde::{Deserialize, Serialize};
use CommandArguments::Queue;

use crate::commands::QueueCommandArguments::Remove;
use crate::commands::*;
use crate::options::*;

/// Options for the [`QueueAddCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueueRemoveArgs {
    /// A torrent hash
    #[arg(value_name = "HASH")]
    pub queue_rm_hash: Option<String>,
}

#[injectable]
impl QueueRemoveArgs {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for QueueRemoveArgs {
    fn merge(&mut self, _alternative: &Self) {}

    fn apply_defaults(&mut self) {}

    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if let Some(hash) = &self.queue_rm_hash {
            if Hash::<20>::from_string(hash).is_err() {
                errors.push(HashInvalid("Queue remove hash".to_owned(), hash.to_owned()));
            }
        } else {
            errors.push(NotSet("Queue remove hash".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Queue {
                command: Remove { args, .. },
                ..
            }) => Some(args),
            _ => None,
        }
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for QueueRemoveArgs {
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
