use crate::naming::FORBIDDEN_CHARACTERS;
use crate::source::{Metadata, Source};

pub struct SourceName;

impl SourceName {
    #[must_use]
    pub fn get(source: &Source) -> String {
        Self::from_metadata(&source.metadata)
    }

    #[must_use]
    pub fn get_escaped(source: &Source) -> String {
        Self::get(source).replace(&FORBIDDEN_CHARACTERS[..], "_")
    }

    #[must_use]
    pub fn from_metadata(metadata: &Metadata) -> String {
        if metadata.remaster_title.is_empty() {
            format!(
                "{} - {} [{}]",
                metadata.artist, metadata.album, metadata.year
            )
        } else {
            format!(
                "{} - {} ({}) [{}]",
                metadata.artist, metadata.album, metadata.remaster_title, metadata.year
            )
        }
    }
}
