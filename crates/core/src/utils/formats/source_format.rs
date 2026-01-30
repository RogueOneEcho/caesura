use crate::prelude::*;
use SourceFormat::*;
/// Format of a [Source].
#[derive(Clone, Copy, Debug)]
pub enum SourceFormat {
    Flac24,
    Flac,
}

impl SourceFormat {
    /// Display name of the format.
    #[must_use]
    pub fn get_name(&self) -> &str {
        match self {
            Flac24 => "FLAC 24bit",
            Flac => "FLAC",
        }
    }

    /// Convert to the equivalent [`ExistingFormat`].
    #[must_use]
    pub fn to_existing(self) -> ExistingFormat {
        match self {
            Flac24 => ExistingFormat::Flac24,
            Flac => ExistingFormat::Flac,
        }
    }

    /// Full title for the API.
    #[must_use]
    pub fn get_title(&self) -> &str {
        match self {
            Flac24 => "FLAC 24bit Lossless",
            Flac => "FLAC Lossless",
        }
    }
}

impl Display for SourceFormat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.get_name())
    }
}
