use std::fmt::{Display, Formatter};

use clap::Args;
use di::injectable;
use serde::{Deserialize, Serialize};

use crate::commands::*;
use crate::options::*;

use crate::commands::CommandArguments::*;

/// Source argument used by Verify, Spectrogram, Transcode, and Upload commands
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SourceArg {
    /// Source as: torrent id, path to torrent file, or indexer url.
    ///
    /// Examples:
    /// `4871992`,
    /// `path/to/something.torrent`,
    /// `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or
    /// `https://example.com/torrents.php?torrentid=4871992`
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,
}

#[injectable]
impl SourceArg {
    fn new() -> Self {
        Self::from_args().unwrap_or_default()
    }

    /// Get from command line arguments.
    #[must_use]
    pub fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(
                Spectrogram { source, .. }
                | Transcode { source, .. }
                | Verify { source, .. }
                | Upload { source, .. },
            ) => Some(source),
            _ => None,
        }
    }

    /// Validate the source argument.
    #[must_use]
    pub fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if self.source.is_none() {
            errors.push(NotSet("Source".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }
}

impl Display for SourceArg {
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
