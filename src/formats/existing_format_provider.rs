use std::collections::HashSet;

use crate::api::Torrent;
use crate::errors::AppError;
use crate::formats::ExistingFormat;

pub struct ExistingFormatProvider;

impl ExistingFormatProvider {
    pub fn get(
        source_torrent: &Torrent,
        group_torrents: &[Torrent],
    ) -> Result<HashSet<ExistingFormat>, AppError> {
        group_torrents
            .iter()
            .filter(|&other_torrent| is_alternative_format(source_torrent, other_torrent))
            .map(Torrent::get_format)
            .collect::<Result<HashSet<_>, _>>()
    }
}

/// Determine if [target] is an alternative format of the [source] release.
fn is_alternative_format(source: &Torrent, target: &Torrent) -> bool {
    target.remaster_title == source.remaster_title
        && target.remaster_record_label == source.remaster_record_label
        && target.media == source.media
        && target.remaster_catalogue_number == source.remaster_catalogue_number
}
