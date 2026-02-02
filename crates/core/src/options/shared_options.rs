use crate::commands::CommandArguments::{self, *};
use crate::commands::QueueCommandArguments;
use crate::prelude::*;
use caesura_macros::Options;
use rogue_logging::{TimeFormat, Verbosity};
use serde::{Deserialize, Serialize};

/// Options shared by all commands
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SharedOptions {
    /// Announce URL including passkey
    ///
    /// Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
    #[arg(long)]
    #[options(required)]
    pub announce_url: String,

    /// API key with torrent permissions for the indexer.
    #[arg(long)]
    #[options(required)]
    pub api_key: String,

    /// ID of the tracker as it appears in the source field of a torrent.
    ///
    /// Examples: `red`, `pth`, `ops`
    #[arg(long)]
    #[options(required, default_fn = default_indexer, default_doc = "from announce_url")]
    pub indexer: String,

    /// URL of the indexer.
    ///
    /// Examples: `https://redacted.sh`, `https://orpheus.network`
    #[arg(long)]
    #[options(required, default_fn = default_indexer_url, default_doc = "from announce_url")]
    pub indexer_url: String,

    /// Directories containing torrent content.
    ///
    /// Typically this is set as the download directory in your torrent client.
    #[arg(long)]
    #[options(default = vec![PathBuf::from("./content")])]
    pub content: Vec<PathBuf>,

    /// Level of logs to display.
    #[arg(long, value_enum)]
    pub verbosity: Verbosity,

    /// Path to the configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Time format to use in logs.
    #[arg(long)]
    pub log_time: TimeFormat,

    /// Directory where transcodes and spectrograms will be written.
    #[arg(long)]
    #[options(default = PathBuf::from("./output"))]
    pub output: PathBuf,
}

fn default_indexer(partial: &SharedOptionsPartial) -> Option<String> {
    match partial.announce_url.as_deref() {
        Some(url) if url.starts_with("https://flacsfor.me") => Some("red".to_owned()),
        Some(url) if url.starts_with("https://home.opsfet.ch") => Some("ops".to_owned()),
        _ => None,
    }
}

fn default_indexer_url(partial: &SharedOptionsPartial) -> Option<String> {
    let indexer = partial.indexer.clone().or_else(|| default_indexer(partial));
    match indexer.as_deref() {
        Some("red") => Some("https://redacted.sh".to_owned()),
        Some("ops") => Some("https://orpheus.network".to_owned()),
        _ => None,
    }
}

impl SharedOptions {
    /// Default indexer used by [`Self::mock()`] for testing.
    #[cfg(test)]
    pub const MOCK_INDEXER: &'static str = "red";

    /// Returns the indexer name, normalized to lowercase.
    #[must_use]
    pub fn indexer_lowercase(&self) -> String {
        self.indexer.to_lowercase()
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            indexer: Self::MOCK_INDEXER.to_owned(),
            indexer_url: "https://redacted.sh".to_owned(),
            announce_url: "https://flacsfor.me/test/announce".to_owned(),
            api_key: "test_api_key".to_owned(),
            ..SharedOptions::default()
        }
    }
}

impl OptionsContract for SharedOptions {
    type Partial = SharedOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
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
        if !self.indexer_url.starts_with("https://") && !self.indexer_url.starts_with("http://") {
            errors.push(UrlNotHttp(
                "Indexer URL".to_owned(),
                self.indexer_url.clone(),
            ));
        }
        if self.indexer_url.ends_with('/') {
            errors.push(UrlInvalidSuffix(
                "Indexer URL".to_owned(),
                self.indexer_url.clone(),
            ));
        }
        if !self.announce_url.starts_with("https://") && !self.announce_url.starts_with("http://") {
            errors.push(UrlNotHttp(
                "Announce URL".to_owned(),
                self.announce_url.clone(),
            ));
        }
        if self.announce_url.ends_with('/') {
            errors.push(UrlInvalidSuffix(
                "Announce URL".to_owned(),
                self.announce_url.clone(),
            ));
        }
        if self.content.is_empty() {
            errors.push(IsEmpty("Content Directory".to_owned()));
        }
        for dir in &self.content {
            if !dir.exists() || !dir.is_dir() {
                errors.push(DoesNotExist(
                    "Content Directory".to_owned(),
                    dir.to_string_lossy().to_string(),
                ));
            }
        }
        if !self.output.exists() || !self.output.is_dir() {
            errors.push(DoesNotExist(
                "Output Directory".to_owned(),
                self.output.to_string_lossy().to_string(),
            ));
        }
    }
}

impl FromArgs for SharedOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(
                Batch { shared, .. }
                | Spectrogram { shared, .. }
                | Transcode { shared, .. }
                | Upload { shared, .. }
                | Verify { shared, .. },
            ) => Some(shared.clone()),
            Some(CommandArguments::Queue { command }) => match command {
                QueueCommandArguments::Add { shared, .. }
                | QueueCommandArguments::List { shared, .. }
                | QueueCommandArguments::Remove { shared, .. }
                | QueueCommandArguments::Summary { shared, .. } => Some(shared.clone()),
            },
            _ => None,
        }
    }
}
