use ExistingFormat::*;
use clap::ValueEnum;
use colored::Colorize;
use gazelle_api::Torrent;
use log::trace;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use crate::utils::*;
/// Format of an existing release.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ExistingFormat {
    Flac24 = 0,
    Flac = 1,
    #[serde(rename = "320")]
    _320 = 2,
    V0 = 3,
}

impl ExistingFormat {
    /// Determine format from torrent metadata.
    pub fn from_torrent(torrent: &Torrent) -> Option<ExistingFormat> {
        match (torrent.format.as_str(), torrent.encoding.as_str()) {
            ("FLAC", "Lossless") => Some(Flac),
            ("FLAC", "24bit Lossless") => Some(Flac24),
            ("MP3", "320") => Some(_320),
            ("MP3", "V0 (VBR)") => Some(V0),
            (format, encoding) => {
                trace!(
                    "{} to determine ExistingFormat of `{format}` with encoding `{encoding}`",
                    "Failed".bold()
                );
                None
            }
        }
    }

    /// Display name of the format.
    #[must_use]
    pub fn get_name(&self) -> &str {
        match self {
            Flac24 => "FLAC 24bit",
            Flac => "FLAC",
            _320 => "320",
            V0 => "V0",
        }
    }

    /// Convert to [`SourceFormat`] if this is a lossless format.
    pub fn to_source(self) -> Option<SourceFormat> {
        match self {
            Flac24 => Some(SourceFormat::Flac24),
            Flac => Some(SourceFormat::Flac),
            _ => None,
        }
    }
}

impl Display for ExistingFormat {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.get_name())
    }
}

impl PartialOrd for ExistingFormat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExistingFormat {
    #[allow(clippy::as_conversions)]
    fn cmp(&self, other: &Self) -> Ordering {
        let left = *self as isize;
        let right = *other as isize;
        left.cmp(&right)
    }
}
