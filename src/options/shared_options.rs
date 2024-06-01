use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::Args;
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::logging::{Info, Verbosity};
use crate::options::{
    DoesNotExist, NotSet, OptionRule, Options, OptionsProvider, UrlInvalidSuffix, UrlNotHttp,
};

/// Options for all commands
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SharedOptions {
    /// API key
    #[arg(long)]
    pub api_key: Option<String>,

    /// ID of the tracker as it appears in the source field of a torrent.
    /// Examples: red, pth, ops;
    /// Default: red
    #[arg(long)]
    pub indexer: Option<String>,

    #[allow(clippy::doc_markdown)]
    /// URL of the indexer.
    /// Examples: https://redacted.ch, https://orpheus.network;
    /// Default: Dependent on indexer
    #[arg(long)]
    pub indexer_url: Option<String>,

    #[allow(clippy::doc_markdown)]
    /// URL of the tracker.
    /// Examples: https://flacsfor.me, https://home.opsfet.ch;
    /// Default: Dependent on indexer
    #[arg(long)]
    pub tracker_url: Option<String>,

    /// Directory containing torrent content.
    /// Typically this is set as the download directory in your torrent client.
    #[arg(long)]
    pub content_directory: Option<PathBuf>,

    /// Number of cpus to use for processing.
    /// Default: Total number of CPUs
    #[arg(long)]
    pub cpu_limit: Option<u16>,

    /// Level of logs to display.
    /// Default: info
    #[arg(long, value_enum)]
    pub verbosity: Option<Verbosity>,

    /// Path to the configuration file.
    /// Default: config.json (in current working directory)
    #[arg(long)]
    pub config_path: Option<PathBuf>,

    #[allow(clippy::doc_markdown)]
    /// Source as: torrent id, path to torrent file, or indexer url.
    ///
    /// Examples:
    /// 4871992,
    /// path/to/something.torrent,
    /// https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992, or
    /// https://example.com/torrents.php?torrentid=4871992
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    /// Directory where transcodes and spectrograms will be written.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[injectable]
impl SharedOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get_shared_options()
    }
}

impl Options for SharedOptions {
    fn get_name() -> String {
        "Shared Options".to_owned()
    }

    /// Merge the current options with an alternative set of options
    ///
    /// The current options will take precedence over the alternative options
    fn merge(&mut self, alternative: &Self) {
        if self.api_key.is_none() {
            self.api_key.clone_from(&alternative.api_key);
        }
        if self.indexer.is_none() {
            self.indexer.clone_from(&alternative.indexer);
        }
        if self.indexer_url.is_none() {
            self.indexer_url.clone_from(&alternative.indexer_url);
        }
        if self.tracker_url.is_none() {
            self.tracker_url.clone_from(&alternative.tracker_url);
        }
        if self.content_directory.is_none() {
            self.content_directory
                .clone_from(&alternative.content_directory);
        }
        if self.cpu_limit.is_none() {
            self.cpu_limit = alternative.cpu_limit;
        }
        if self.verbosity.is_none() {
            self.verbosity = alternative.verbosity;
        }
        if self.config_path.is_none() {
            self.config_path.clone_from(&alternative.config_path);
        }
        if self.source.is_none() {
            self.source.clone_from(&alternative.source);
        }
        if self.output.is_none() {
            self.output.clone_from(&alternative.output);
        }
    }

    fn apply_defaults(&mut self) {
        if self.indexer.is_none() {
            self.indexer = Some("red".to_owned());
        }
        if self.indexer_url.is_none() {
            self.indexer_url = match self.indexer.as_deref() {
                Some("red") => Some("https://redacted.ch".to_owned()),
                Some("ops") => Some("https://orpheus.network".to_owned()),
                _ => None,
            }
        }
        if self.tracker_url.is_none() {
            self.tracker_url = match self.indexer.as_deref() {
                Some("red") => Some("https://flacsfor.me".to_owned()),
                Some("ops") => Some("https://home.opsfet.ch".to_owned()),
                _ => None,
            }
        }
        if self.cpu_limit.is_none() {
            self.cpu_limit = Some(num_cpus::get() as u16);
        }
        if self.verbosity.is_none() {
            self.verbosity = Some(Info);
        }
    }

    /// Validate the options
    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if self.api_key.is_none() {
            errors.push(NotSet("API Key".to_owned()));
        }
        if self.indexer.is_none() {
            errors.push(NotSet("Indexer".to_owned()));
        }
        if self.indexer_url.is_none() {
            errors.push(NotSet("Indexer URL".to_owned()));
        } else {
            let indexer_url = self.indexer_url.as_ref().expect("Options should be set");
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
        if self.tracker_url.is_none() {
            errors.push(NotSet("Tracker URL".to_owned()));
        } else {
            let tracker_url = self.tracker_url.as_ref().expect("Options should be set");
            if !tracker_url.starts_with("https://") && !tracker_url.starts_with("http://") {
                errors.push(UrlNotHttp("Tracker URL".to_owned(), tracker_url.clone()));
            }
            if tracker_url.ends_with('/') {
                errors.push(UrlInvalidSuffix(
                    "Tracker URL".to_owned(),
                    tracker_url.clone(),
                ));
            }
        }
        if let Some(content_directory) = &self.content_directory {
            if !content_directory.exists() && !content_directory.is_dir() {
                errors.push(DoesNotExist(
                    "Content Directory".to_owned(),
                    content_directory.to_string_lossy().to_string(),
                ));
            }
        } else {
            errors.push(NotSet("Content Directory".to_owned()));
        }
        if let Some(config_path) = &self.config_path {
            if !config_path.exists() && !config_path.is_file() {
                errors.push(DoesNotExist(
                    "Config File".to_owned(),
                    config_path.to_string_lossy().to_string(),
                ));
            }
        }
        if self.source.is_none() {
            errors.push(NotSet("Source".to_owned()));
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

    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }
}

impl Display for SharedOptions {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(json) = serde_json::to_string_pretty(self) {
            json
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
