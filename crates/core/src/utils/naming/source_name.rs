use crate::prelude::*;

/// Generate directory names for sources.
pub struct SourceName;

impl SourceName {
    /// Sanitized directory name for a source.
    #[must_use]
    pub fn get(metadata: &Metadata) -> String {
        let name = Self::get_unsanitized(metadata);
        Sanitizer::execute(name)
    }

    /// Directory name for a source without sanitization.
    #[must_use]
    pub fn get_unsanitized(metadata: &Metadata) -> String {
        match &metadata.edition_title {
            Some(title) => format!(
                "{} - {} ({}) [{}]",
                metadata.artist, metadata.album, title, metadata.year
            ),
            None => format!(
                "{} - {} [{}]",
                metadata.artist, metadata.album, metadata.year
            ),
        }
    }
}
