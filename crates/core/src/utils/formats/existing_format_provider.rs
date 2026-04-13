use crate::prelude::*;
use gazelle_api::Torrent;

/// Determine which formats already exist for a given release.
pub struct ExistingFormatProvider;

impl ExistingFormatProvider {
    /// Existing formats matching the source release in the torrent group.
    pub fn get(source_torrent: &Torrent, group_torrents: &[Torrent]) -> BTreeSet<ExistingFormat> {
        let source_key = EditionKey::from_torrent(source_torrent);
        group_torrents
            .iter()
            .filter(|&other_torrent| EditionKey::from_torrent(other_torrent) == source_key)
            .filter_map(ExistingFormat::from_torrent)
            .collect()
    }
}
