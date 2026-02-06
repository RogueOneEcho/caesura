use crate::prelude::*;
use gazelle_api::{Group, Torrent};
use rogue_logging::Colors;

/// Source to be transcoded
#[derive(Debug)]
pub struct Source {
    pub torrent: Torrent,

    pub group: Group,

    pub existing: BTreeSet<ExistingFormat>,

    pub format: SourceFormat,

    pub directory: PathBuf,

    pub metadata: Metadata,
}

impl Display for Source {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{}",
            SourceName::get(&self.metadata).gray().italic()
        )
    }
}
