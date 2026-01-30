use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{Batch, Queue, Spectrogram, Transcode, Upload, Verify};
use crate::commands::QueueCommandArguments::{Add, List, Remove, Summary};
use crate::commands::*;
use crate::options::*;
use caesura_macros::Options;
use rogue_logging::{TimeFormat, Verbosity};

/// Options shared by all commands
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Queue, Spectrogram, Transcode, Verify, Upload))]
#[options(from_args_fn = "Self::partial_from_args")]
#[options(defaults_fn = "Self::apply_calculated_defaults")]
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
    #[options(default = vec![PathBuf::from("./content")])]
    pub content: Vec<PathBuf>,

    /// Level of logs to display.
    ///
    /// Default: `info`
    #[arg(long, value_enum)]
    pub verbosity: Verbosity,

    /// Path to the configuration file.
    ///
    /// Default: `./config.yml`
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Time format to use in logs.
    ///
    /// Default: `Local`
    #[arg(long)]
    pub log_time: TimeFormat,

    /// Directory where transcodes and spectrograms will be written.
    ///
    /// Default: `./output`
    #[arg(long)]
    #[options(default = PathBuf::from("./output"))]
    pub output: PathBuf,
}

impl SharedOptions {
    /// Apply calculated defaults that depend on runtime values.
    pub fn apply_calculated_defaults(partial: &mut SharedOptionsPartial) {
        // indexer is calculated from announce_url
        if partial.indexer.is_none() {
            partial.indexer = match partial.announce_url.as_deref() {
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
        // indexer_url is calculated from indexer
        if partial.indexer_url.is_none() {
            partial.indexer_url = match partial.indexer.as_deref() {
                Some("red") => Some("https://redacted.sh".to_owned()),
                Some("ops") => Some("https://orpheus.network".to_owned()),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            indexer: Some("red".to_owned()),
            indexer_url: Some("https://redacted.sh".to_owned()),
            announce_url: Some("https://flacsfor.me/test/announce".to_owned()),
            api_key: Some("test_api_key".to_owned()),
            ..SharedOptions::default()
        }
    }
}

/// Manual `Default` impl with values matching the `#[options(default = ...)]` attributes.
/// Using `#[derive(Default)]` would give empty values for `content` and `output`.
impl Default for SharedOptions {
    fn default() -> Self {
        Self {
            announce_url: None,
            api_key: None,
            indexer: None,
            indexer_url: None,
            content: vec![PathBuf::from("./content")],
            verbosity: Verbosity::default(),
            config: None,
            log_time: TimeFormat::default(),
            output: PathBuf::from("./output"),
        }
    }
}

impl SharedOptions {
    /// Custom `from_args` implementation for complex Queue subcommand matching
    pub fn partial_from_args() -> Option<SharedOptionsPartial> {
        match ArgumentsParser::get() {
            Some(
                Batch { shared, .. }
                | Queue {
                    command:
                        Add { shared, .. }
                        | List { shared, .. }
                        | Summary { shared, .. }
                        | Remove { shared, .. },
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

    /// Validate the partial options.
    pub fn validate_partial(partial: &SharedOptionsPartial, errors: &mut Vec<OptionRule>) {
        if let Some(config) = &partial.config {
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
        if partial.api_key.is_none() {
            errors.push(NotSet("API Key".to_owned()));
        }
        if partial.indexer.is_none() {
            errors.push(NotSet("Indexer".to_owned()));
        }
        if let Some(indexer_url) = &partial.indexer_url {
            if !indexer_url.starts_with("https://") && !indexer_url.starts_with("http://") {
                errors.push(UrlNotHttp("Indexer URL".to_owned(), indexer_url.clone()));
            }
            if indexer_url.ends_with('/') {
                errors.push(UrlInvalidSuffix(
                    "Indexer URL".to_owned(),
                    indexer_url.clone(),
                ));
            }
        } else {
            errors.push(NotSet("Indexer URL".to_owned()));
        }
        if let Some(announce_url) = &partial.announce_url {
            if !announce_url.starts_with("https://") && !announce_url.starts_with("http://") {
                errors.push(UrlNotHttp("Announce URL".to_owned(), announce_url.clone()));
            }
            if announce_url.ends_with('/') {
                errors.push(UrlInvalidSuffix(
                    "Announce URL".to_owned(),
                    announce_url.clone(),
                ));
            }
        } else {
            errors.push(NotSet("Announce URL".to_owned()));
        }
        if let Some(directories) = &partial.content {
            if directories.is_empty() {
                errors.push(IsEmpty("Content Directory".to_owned()));
            }
            for dir in directories {
                if !dir.exists() || !dir.is_dir() {
                    errors.push(DoesNotExist(
                        "Content Directory".to_owned(),
                        dir.to_string_lossy().to_string(),
                    ));
                }
            }
        }
        if let Some(output_directory) = &partial.output
            && (!output_directory.exists() || !output_directory.is_dir())
        {
            errors.push(DoesNotExist(
                "Output Directory".to_owned(),
                output_directory.to_string_lossy().to_string(),
            ));
        }
    }
}
