use crate::prelude::*;

/// Source argument used by Verify, Spectrogram, Transcode, and Upload commands
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SourceArg {
    /// Source as: torrent id, path to torrent file, indexer url, or 40-character info hash.
    ///
    /// Examples:
    /// `4871992`,
    /// `path/to/something.torrent`,
    /// `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`,
    /// `https://example.com/torrents.php?torrentid=4871992`, or
    /// `0123456789abcdef0123456789abcdef01234567`
    #[arg(value_name = "SOURCE")]
    pub source: String,
}

impl OptionsContract for SourceArg {
    type Partial = SourceArgPartial;

    fn validate(&self, _validator: &mut OptionsValidator) {}
}

impl Display for SourceArg {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = if let Ok(yaml) = yaml_to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        write!(formatter, "{output}")
    }
}
