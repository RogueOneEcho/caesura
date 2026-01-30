use std::fmt::{Display, Formatter};

use CommandArguments::Queue;
use clap::Args;
use di::injectable;
use flat_db::Hash;
use serde::{Deserialize, Serialize};

use crate::commands::QueueCommandArguments::Remove;
use crate::commands::*;
use crate::options::*;

/// Options for the [`QueueRemoveCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueueRemoveArgs {
    /// A torrent hash
    #[arg(value_name = "HASH")]
    pub queue_rm_hash: Option<String>,
}

#[injectable]
impl QueueRemoveArgs {
    fn new() -> Self {
        Self::from_args().unwrap_or_default()
    }

    /// Get from command line arguments.
    #[allow(clippy::match_wildcard_for_single_variants)]
    #[must_use]
    pub fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Queue {
                command: Remove { args, .. },
                ..
            }) => Some(args),
            _ => None,
        }
    }

    /// Validate the queue remove arguments.
    #[must_use]
    pub fn validate(&self) -> bool {
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
