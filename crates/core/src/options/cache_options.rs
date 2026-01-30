use crate::commands::CommandArguments::{Batch, Queue};
use crate::commands::QueueCommandArguments::*;
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for queue cache
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Queue))]
#[options(from_args_fn = "Self::partial_from_args")]
pub struct CacheOptions {
    /// Path to cache directory.
    ///
    /// Default: `./cache`
    #[arg(long)]
    #[options(default = PathBuf::from("./cache"))]
    pub cache: PathBuf,
}

impl CacheOptions {
    /// Custom `from_args` implementation for complex Queue subcommand matching
    #[must_use]
    pub fn partial_from_args() -> Option<CacheOptionsPartial> {
        match ArgumentsParser::get() {
            Some(
                Batch { cache, .. }
                | Queue {
                    command:
                        Add { cache, .. }
                        | List { cache, .. }
                        | Summary { cache, .. }
                        | Remove { cache, .. },
                },
            ) => Some(cache),
            _ => None,
        }
    }

    /// Validate the partial options.
    pub fn validate_partial(partial: &CacheOptionsPartial, errors: &mut Vec<OptionRule>) {
        let default_cache = PathBuf::from("./cache");
        if let Some(cache) = &partial.cache {
            if cache.ends_with(".json") || (cache.eq(&default_cache) && !cache.is_dir()) {
                errors.push(Changed(
                    "Cache Directory".to_owned(),
                    cache.to_string_lossy().to_string(),
                    "In v0.19.0 the cache format changed. A directory is now required.
Please see the release notes for more details:
https://github.com/RogueOneEcho/caesura/releases/tag/v0.19.0"
                        .to_owned(),
                ));
            }

            if !cache.is_dir() {
                errors.push(DoesNotExist(
                    "Cache Directory".to_owned(),
                    cache.to_string_lossy().to_string(),
                ));
            }
        }
    }
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            cache: PathBuf::from("./cache"),
        }
    }
}
