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
    /// Target formats to transcode to.
    pub targets: BTreeSet<TargetFormat>,
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

impl Source {
    #[cfg(test)]
    pub fn mock() -> Self {
        let group = Group::mock();
        let torrent = Torrent::mock();
        let metadata = Metadata::new(&Group::mock(), &Torrent::mock());
        let url = get_permalink(RED_URL, group.id, torrent.id);
        Self {
            torrent,
            group,
            targets: TargetFormat::all(),
            format: SourceFormat::Flac24,
            directory: PathBuf::from("/path/to/content"),
            metadata,
            url,
        }
    }
}
