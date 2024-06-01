use crate::api::Torrent;
use crate::errors::AppError;
use crate::formats::SourceFormat;

/// Format of an existing release.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ExistingFormat {
    Flac24,
    Flac,
    _320,
    V0,
}

impl ExistingFormat {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub fn to_source(&self) -> Result<SourceFormat, AppError> {
        match self {
            ExistingFormat::Flac24 => Ok(SourceFormat::Flac24),
            ExistingFormat::Flac => Ok(SourceFormat::Flac),
            _ => AppError::explained("get source format", "Format is not FLAC".to_owned()),
        }
    }
}

impl Torrent {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub fn get_format(&self) -> Result<ExistingFormat, AppError> {
        match self.format.as_str() {
            "FLAC" => match self.encoding.as_str() {
                "Lossless" => Ok(ExistingFormat::Flac),
                "24bit Lossless" => Ok(ExistingFormat::Flac24),
                _ => AppError::explained(
                    "get format",
                    format!("unknown encoding: {}", self.encoding),
                ),
            },
            "MP3" => match self.encoding.as_str() {
                "320" => Ok(ExistingFormat::_320),
                "V0 (VBR)" => Ok(ExistingFormat::V0),
                _ => AppError::explained(
                    "get format",
                    format!("unknown encoding: {}", self.encoding),
                ),
            },
            _ => AppError::explained("get format", format!("unknown format: {}", self.format)),
        }
    }
}
