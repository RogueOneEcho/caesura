use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::Args;
use di::{Ref, injectable};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{Batch, Queue};
use crate::commands::QueueCommandArguments::*;

use crate::commands::*;
use crate::options::*;

const DEFAULT_CACHE_PATH: &str = "./cache";

/// Options for [`Queue`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct CacheOptions {
    /// Path to cache directory.
    ///
    /// Default: `./cache`
    #[arg(long)]
    pub cache: Option<PathBuf>,
}

#[injectable]
impl CacheOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for CacheOptions {
    fn merge(&mut self, alternative: &Self) {
        if self.cache.is_none() {
            self.cache.clone_from(&alternative.cache);
        }
    }

    fn apply_defaults(&mut self) {
        if self.cache.is_none() {
            self.cache = Some(PathBuf::from(DEFAULT_CACHE_PATH));
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if let Some(cache) = &self.cache {
            if cache.ends_with(".json")
                || (cache.eq(&PathBuf::from(DEFAULT_CACHE_PATH)) && !cache.is_dir())
            {
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
        OptionRule::show(&errors);
        errors.is_empty()
    }

    #[must_use]
    #[allow(clippy::manual_let_else)]
    fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(
                Batch { cache, .. }
                | Queue {
                    command: Add { cache, .. } | List { cache, .. } | Summary { cache, .. },
                },
            ) => Some(cache),
            _ => None,
        }
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for CacheOptions {
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
