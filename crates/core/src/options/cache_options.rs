use crate::commands::CommandArguments::{self, *};
use crate::commands::QueueCommandArguments;
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for queue cache
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct CacheOptions {
    /// Path to cache directory.
    #[arg(long)]
    #[options(default = PathBuf::from("./cache"))]
    pub cache: PathBuf,
}

impl OptionsContract for CacheOptions {
    type Partial = CacheOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        let default_cache = PathBuf::from("./cache");
        if self.cache.ends_with(".json") || (self.cache.eq(&default_cache) && !self.cache.is_dir())
        {
            errors.push(Changed(
                "Cache Directory".to_owned(),
                self.cache.to_string_lossy().to_string(),
                "In v0.19.0 the cache format changed. A directory is now required.
Please see the release notes for more details:
https://github.com/RogueOneEcho/caesura/releases/tag/v0.19.0"
                    .to_owned(),
            ));
        }
        if !self.cache.is_dir() {
            errors.push(DoesNotExist(
                "Cache Directory".to_owned(),
                self.cache.to_string_lossy().to_string(),
            ));
        }
    }
}

impl FromArgs for CacheOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { cache, .. }) => Some(cache.clone()),
            Some(CommandArguments::Queue { command }) => match command {
                QueueCommandArguments::Add { cache, .. }
                | QueueCommandArguments::List { cache, .. }
                | QueueCommandArguments::Remove { cache, .. }
                | QueueCommandArguments::Summary { cache, .. } => Some(cache.clone()),
            },
            _ => None,
        }
    }
}
