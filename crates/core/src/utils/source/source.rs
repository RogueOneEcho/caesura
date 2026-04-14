use crate::prelude::*;
use gazelle_api::{Group, Torrent};
use rogue_logging::Colors;

/// Source to be transcoded
#[derive(Debug)]
pub struct Source {
    /// Torrent metadata from the tracker API.
    pub torrent: Torrent,
    /// Torrent group metadata from the tracker API.
    pub group: Group,
    /// Formats that already exist for this torrent group.
    pub existing: BTreeSet<ExistingFormat>,
    /// Audio format of the source files.
    pub format: SourceFormat,
    /// Path to the source FLAC directory.
    pub directory: PathBuf,
    /// Audio metadata extracted from the source files.
    pub metadata: Metadata,
    /// Permalink URL to the torrent on the tracker.
    pub url: String,
}

impl Display for Source {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let text = SourceName::get(&self.metadata).gray().italic();
        write!(formatter, "{}", text.hyperlink(&self.url))
    }
}
