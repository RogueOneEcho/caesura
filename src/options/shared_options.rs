use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::Args;
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::*;
use crate::commands::QueueCommandArguments::*;

use rogue_logging::{TimeFormat, Verbosity};

use crate::commands::CommandArguments::{Batch, Queue};
use crate::commands::*;
use crate::options::*;
pub const DEFAULT_CONFIG_PATH: &str = "config.yml";
const DEFAULT_CONTENT_PATH: &str = "./content";
const DEFAULT_OUTPUT_PATH: &str = "./output";

/// Options shared by all commands
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SharedOptions {
    /// Announce URL including passkey
    ///
    /// Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
    #[arg(long)]
    pub announce_url: Option<String>,

    /// API key with torrent permissions for the indexer.
    #[arg(long)]
    pub api_key: Option<String>,

    /// ID of the tracker as it appears in the source field of a torrent.
    ///
    /// Examples: `red`, `pth`, `ops`
    ///
    /// Default: Determined by `announce_url`
    #[arg(long)]
    pub indexer: Option<String>,

    /// URL of the indexer.
    ///
    /// Examples: `https://redacted.sh`, `https://orpheus.network`
    ///
    /// Default: Determined by `announce_url`
    #[arg(long)]
    pub indexer_url: Option<String>,

    /// Directories containing torrent content.
    ///
    /// Typically this is set as the download directory in your torrent client.
    ///
    /// Default: `./content`
    #[arg(long)]
    pub content: Option<Vec<PathBuf>>,

    /// Level of logs to display.
    ///
    /// Default: `info`
    #[arg(long, value_enum)]
    pub verbosity: Option<Verbosity>,

    /// Path to the configuration file.
    ///
    /// Default: `./config.yml`
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Time format to use in logs.
    ///
    /// Default: `datetime`
    #[arg(long)]
    pub log_time: Option<TimeFormat>,

    /// Directory where transcodes and spectrograms will be written.
    ///
    /// Default: `./output`
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[injectable]
impl SharedOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for SharedOptions {
    fn get_name() -> String {
        "Shared Options".to_owned()
    }

    fn merge(&mut self, alternative: &Self) {
        if self.announce_url.is_none() {
            self.announce_url.clone_from(&alternative.announce_url);
        }
        if self.api_key.is_none() {
            self.api_key.clone_from(&alternative.api_key);
        }
        if self.indexer.is_none() {
            self.indexer.clone_from(&alternative.indexer);
        }
        if self.indexer_url.is_none() {
            self.indexer_url.clone_from(&alternative.indexer_url);
        }
        if self.content.is_none() {
            self.content.clone_from(&alternative.content);
        }
        if self.verbosity.is_none() {
            self.verbosity = alternative.verbosity;
        }
        if self.config.is_none() {
            self.config.clone_from(&alternative.config);
        }
        if self.log_time.is_none() {
            self.log_time.clone_from(&alternative.log_time);
        }
        if self.output.is_none() {
            self.output.clone_from(&alternative.output);
        }
    }

    fn apply_defaults(&mut self) {
        if self.indexer.is_none() {
            self.indexer = match self.announce_url.as_deref() {
                Some(url) => {
                    if url.starts_with("https://flacsfor.me") {
                        Some("red".to_owned())
                    } else if url.starts_with("https://home.opsfet.ch") {
                        Some("ops".to_owned())
                    } else {
                        None
                    }
                }
                _ => None,
            };
        }
        if self.indexer_url.is_none() {
            self.indexer_url = match self.indexer.as_deref() {
                Some("red") => Some("https://redacted.sh".to_owned()),
                Some("ops") => Some("https://orpheus.network".to_owned()),
                _ => None,
            }
        }
        if self.verbosity.is_none() {
            self.verbosity = Some(Verbosity::default());
        }
        if self.log_time.is_none() {
            self.log_time = Some(TimeFormat::default());
        }
        if self.content.is_none() {
            self.content = Some(vec![PathBuf::from(DEFAULT_CONTENT_PATH)]);
        }
        if self.output.is_none() {
            self.output = Some(PathBuf::from(DEFAULT_OUTPUT_PATH));
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if let Some(config) = &self.config {
            if config.ends_with(".json")
                || (config.eq(&PathBuf::from(DEFAULT_CONFIG_PATH)) && !config.is_file())
            {
                errors.push(Changed(
                    "Config File".to_owned(),
                    config.to_string_lossy().to_string(),
                    "In v0.19.0 the config file format changed. A YAML file is now required.
Please see the release notes for more details:
https://github.com/RogueOneEcho/caesura/releases/tag/v0.19.0"
                        .to_owned(),
                ));
            }
            if !config.is_file() {
                errors.push(DoesNotExist(
                    "Config File".to_owned(),
                    config.to_string_lossy().to_string(),
                ));
            }
        }
        if self.api_key.is_none() {
            errors.push(NotSet("API Key".to_owned()));
        }
        if self.indexer.is_none() {
            errors.push(NotSet("Indexer".to_owned()));
        }
        if self.indexer_url.is_none() {
            errors.push(NotSet("Indexer URL".to_owned()));
        } else {
            let indexer_url = self.indexer_url.clone().expect("indexer_url should be set");
            if !indexer_url.starts_with("https://") && !indexer_url.starts_with("http://") {
                errors.push(UrlNotHttp("Indexer URL".to_owned(), indexer_url.clone()));
            }
            if indexer_url.ends_with('/') {
                errors.push(UrlInvalidSuffix(
                    "Indexer URL".to_owned(),
                    indexer_url.clone(),
                ));
            }
        }
        if self.announce_url.is_none() {
            errors.push(NotSet("Announce URL".to_owned()));
        } else {
            let announce_url = self
                .announce_url
                .clone()
                .expect("announce_url should be set");
            if !announce_url.starts_with("https://") && !announce_url.starts_with("http://") {
                errors.push(UrlNotHttp("Announce URL".to_owned(), announce_url.clone()));
            }
            if announce_url.ends_with('/') {
                errors.push(UrlInvalidSuffix(
                    "Announce URL".to_owned(),
                    announce_url.clone(),
                ));
            }
        }
        if let Some(directories) = &self.content {
            for dir in directories {
                if !dir.exists() || !dir.is_dir() {
                    errors.push(DoesNotExist(
                        "Content Directory".to_owned(),
                        dir.to_string_lossy().to_string(),
                    ));
                }
            }
        } else {
            errors.push(NotSet("Content Directory".to_owned()));
        }
        if let Some(output_directory) = &self.output {
            if !output_directory.exists() || !output_directory.is_dir() {
                errors.push(DoesNotExist(
                    "Output Directory".to_owned(),
                    output_directory.to_string_lossy().to_string(),
                ));
            }
        } else {
            errors.push(NotSet("Output Directory".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(
                Batch { shared, .. }
                | Queue {
                    command: Add { shared, .. } | List { shared, .. } | Summary { shared, .. },
                    ..
                }
                | Spectrogram { shared, .. }
                | Transcode { shared, .. }
                | Verify { shared, .. }
                | Upload { shared, .. },
            ) => Some(shared),
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

impl Display for SharedOptions {
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
