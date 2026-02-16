use crate::commands::CommandArguments::{self, *};
use crate::commands::QueueCommandArguments;
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Options for `queue add` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QueueAddArgs {
    /// A path to either a directory of `.torrent` files or a single YAML queue file.
    ///
    /// Examples: `./torrents`, `/path/to/torrents`, `./queue.yml`
    #[arg(value_name = "PATH")]
    pub queue_add_path: Option<PathBuf>,
}

impl OptionsContract for QueueAddArgs {
    type Partial = QueueAddArgsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(path) = &self.queue_add_path {
            if !path.exists() {
                errors.push(OptionRule::DoesNotExist(
                    "Queue add path".to_owned(),
                    path.to_string_lossy().to_string(),
                ));
            }
        } else {
            errors.push(OptionRule::NotSet("Queue add path".to_owned()));
        }
    }
}

impl FromArgs for QueueAddArgsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Queue {
                command: QueueCommandArguments::Add { args, .. },
                ..
            }) => Some(args.clone()),
            _ => None,
        }
    }
}
