use crate::prelude::*;
use caesura_macros::Options;
use rogue_logging::{TimeFormat, Verbosity};
use serde::{Deserialize, Serialize};

/// Legacy output path from before platform user directories.
const LEGACY_OUTPUT_DIR: &str = "./output";

/// Validation label for the content directory.
pub(crate) const CONTENT_DIR_LABEL: &str = "Content Directory";

/// Validation label for the output directory.
pub(crate) const OUTPUT_DIR_LABEL: &str = "Output Directory";

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
    #[options(required, default_fn = default_content)]
    pub content: Vec<PathBuf>,

    /// Level of logs to display.
    #[arg(long, value_enum)]
    pub verbosity: Verbosity,

    /// Time format to use in logs.
    #[arg(long)]
    pub log_time: TimeFormat,

    /// Directory where transcodes and spectrograms will be written.
    #[arg(long)]
    #[options(default_fn = default_output, default_doc = "`~/.local/share/caesura/output/` or platform equivalent")]
    pub output: PathBuf,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_output(_partial: &SharedOptionsPartial) -> Option<PathBuf> {
    Some(PathManager::default_output_dir())
}

fn default_content(_partial: &SharedOptionsPartial) -> Option<Vec<PathBuf>> {
    is_docker().then(|| vec![PathBuf::from("/content")])
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

    /// Output directory path with tilde expansion applied.
    #[must_use]
    pub fn output_path(&self) -> PathBuf {
        self.output.expand_tilde()
    }

    /// Content directory paths with tilde expansion applied.
    #[must_use]
    pub fn content_paths(&self) -> Vec<PathBuf> {
        self.content.iter().map(ExpandTilde::expand_tilde).collect()
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
            errors.push(IsEmpty(CONTENT_DIR_LABEL.to_owned()));
        }
        for dir in self.content_paths() {
            if !dir.exists() || !dir.is_dir() {
                errors.push(DoesNotExist(
                    CONTENT_DIR_LABEL.to_owned(),
                    dir.to_string_lossy().to_string(),
                ));
            }
        }
        let output = self.output_path();
        if !output.exists() || !output.is_dir() {
            errors.push(DoesNotExist(
                OUTPUT_DIR_LABEL.to_owned(),
                output.to_string_lossy().to_string(),
            ));
            if PathBuf::from(LEGACY_OUTPUT_DIR).is_dir() {
                let default_dir = PathManager::default_output_dir();
                errors.push(Changed(
                    OUTPUT_DIR_LABEL.to_owned(),
                    self.output.to_string_lossy().to_string(),
                    format!("In v0.27.0 the default output path changed to {}.\nPass the option: --output {LEGACY_OUTPUT_DIR} to use the previous output path.", default_dir.display()),
                ));
            }
        }
    }
}
