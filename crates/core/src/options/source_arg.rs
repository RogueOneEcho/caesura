use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Source argument used by Verify, Spectrogram, Transcode, and Upload commands
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SourceArg {
    /// Source as: torrent id, path to torrent file, or indexer url.
    ///
    /// Examples:
    /// `4871992`,
    /// `path/to/something.torrent`,
    /// `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or
    /// `https://example.com/torrents.php?torrentid=4871992`
    #[arg(value_name = "SOURCE")]
    pub source: String,
}

impl OptionsContract for SourceArg {
    type Partial = SourceArgPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}

impl Display for SourceArg {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = if let Ok(yaml) = serde_yaml::to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        write!(formatter, "{output}")
    }
}
