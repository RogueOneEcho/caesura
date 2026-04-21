use crate::prelude::*;

/// Determine which formats already exist for a given release.
#[injectable]
pub struct ExistingFormatProvider {
    /// Target format options.
    pub options: Ref<TargetOptions>,
}

impl ExistingFormatProvider {
    /// Determine existing formats for the source release in the torrent group.
    ///
    /// Performs two passes:
    /// 1. Exact `EditionKey` match (always applied)
    /// 2. Possible duplicate detection: finds formats from editions where the
    ///    source has less specific metadata. Skipped when `allow_less_specific`
    ///    is set.
    pub fn get(
        &self,
        source_torrent: &Torrent,
        group_torrents: &[Torrent],
    ) -> BTreeSet<ExistingFormat> {
        let source_key = EditionKey::from_torrent(source_torrent);
        let mut existing: BTreeSet<ExistingFormat> = group_torrents
            .iter()
            .filter(|&other| EditionKey::from_torrent(other) == source_key)
            .filter_map(ExistingFormat::from_torrent)
            .collect();
        if !self.options.allow_less_specific {
            let possible_dupes: BTreeSet<ExistingFormat> = group_torrents
                .iter()
                .filter(|&other| source_key.is_less_specific_than(&EditionKey::from_torrent(other)))
                .filter_map(ExistingFormat::from_torrent)
                .collect();
            for format in &possible_dupes {
                if !existing.contains(format) {
                    debug!("{} {format} as a possible duplicate", "Excluding".bold(),);
                }
            }
            existing.extend(possible_dupes);
        }
        existing
    }
}
