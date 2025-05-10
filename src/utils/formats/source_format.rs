use SourceFormat::*;
use std::fmt::{Display, Formatter};

use crate::utils::*;
/// Format of a [Source].
#[derive(Clone, Copy, Debug)]
pub enum SourceFormat {
    Flac24,
    Flac,
}

impl SourceFormat {
    #[must_use]
    pub fn get_name(&self) -> &str {
        match self {
            Flac24 => "FLAC 24bit",
            Flac => "FLAC",
        }
    }

    #[must_use]
    pub fn to_existing(self) -> ExistingFormat {
        match self {
            Flac24 => ExistingFormat::Flac24,
            Flac => ExistingFormat::Flac,
        }
    }
    #[must_use]
    pub fn get_title(&self) -> &str {
        match self {
            Flac24 => "FLAC 24bit Lossless",
            Flac => "FLAC Lossless",
        }
    }
}

impl Display for SourceFormat {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.get_name())
    }
}
