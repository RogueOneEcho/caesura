use super::track_info::TrackInfo;
use crate::commands::inspect::format::{DisableStyleGuard, divider};
use crate::prelude::*;

/// Inspect audio file metadata in a directory.
#[injectable]
pub(crate) struct InspectCommand {
    arg: Ref<InspectArg>,
}

impl InspectCommand {
    /// Execute [`InspectCommand`] from the CLI.
    pub(crate) fn execute_cli(&self) -> Result<bool, Failure<InspectAction>> {
        let output = get_details(&self.arg.inspect_path, true)?;
        println!("{output}");
        Ok(true)
    }
}

/// Get track details for a directory of audio files.
///
/// Reads audio properties and tags natively using `lofty`.
/// Auto-detects file format by extension (FLAC and MP3).
pub(crate) fn get_details(dir: &Path, style: bool) -> Result<String, Failure<InspectAction>> {
    let (properties, tags) = get_details_split(dir, style)?;
    let _guard = (!style).then(DisableStyleGuard::new);
    Ok(format!("{properties}{}{tags}", divider()))
}

/// Get track details split into properties table and per-track tags.
pub(crate) fn get_details_split(
    dir: &Path,
    style: bool,
) -> Result<(String, String), Failure<InspectAction>> {
    let _guard = (!style).then(DisableStyleGuard::new);
    let tracks = TrackInfo::read_dir(dir)?;
    let properties = TrackInfo::format_properties_table(&tracks);
    let tags = TrackInfo::format_all_tags(&tracks);
    Ok((properties, tags))
}
