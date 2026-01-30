use crate::prelude::*;
use CommandArguments::Queue;
use QueueCommandArguments::Add;
use clap::Args;
use serde::{Deserialize, Serialize};

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
    fn new() -> Self {
        Self::from_args().unwrap_or_default()
    }

    /// Get from command line arguments.
    #[allow(clippy::match_wildcard_for_single_variants)]
    #[must_use]
    pub fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Queue {
                command: Add { args, .. },
                ..
            }) => Some(args),
            _ => None,
        }
    }

    /// Validate the queue add arguments.
    #[must_use]
    pub fn validate(&self) -> bool {
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
}

impl Display for QueueAddArgs {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = if let Ok(yaml) = serde_yaml::to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        write!(formatter, "{output}")
    }
}
