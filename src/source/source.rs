use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

use colored::Colorize;

use crate::api::{Group, Torrent};
use crate::formats::existing_format::ExistingFormat;
use crate::formats::SourceFormat;
use crate::logging::Colors;
use crate::naming::SourceName;
use crate::source::metadata::Metadata;

/// Source to be transcoded
#[derive(Debug)]
pub struct Source {
    pub torrent: Torrent,

    pub group: Group,

    pub existing: HashSet<ExistingFormat>,

    pub format: SourceFormat,

    pub directory: PathBuf,

    pub metadata: Metadata,
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        SourceName::get(self).gray().italic().fmt(formatter)
    }
}
