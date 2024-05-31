use crate::api::Torrent;
use crate::formats::FormatError::*;
use crate::formats::{FormatError, SourceFormat};

/// Format of an existing release.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ExistingFormat {
    Flac24,
    Flac,
    _320,
    V0,
}

impl ExistingFormat {
    #[must_use]
    pub fn to_source(&self) -> Option<SourceFormat> {
        match self {
            ExistingFormat::Flac24 => Some(SourceFormat::Flac24),
            ExistingFormat::Flac => Some(SourceFormat::Flac),
            ExistingFormat::_320 => None,
            ExistingFormat::V0 => None,
        }
    }
}

impl Torrent {
    pub fn get_format(&self) -> Result<ExistingFormat, FormatError> {
        match self.format.as_str() {
            "FLAC" => match self.encoding.as_str() {
                "Lossless" => Ok(ExistingFormat::Flac),
                "24bit Lossless" => Ok(ExistingFormat::Flac24),
                _ => Err(UnknownEncoding(self.encoding.clone())),
            },
            "MP3" => match self.encoding.as_str() {
                "320" => Ok(ExistingFormat::_320),
                "V0 (VBR)" => Ok(ExistingFormat::V0),
                _ => Err(UnknownEncoding(self.encoding.clone())),
            },
            _ => Err(UnknownFormat(self.format.clone())),
        }
    }
}
