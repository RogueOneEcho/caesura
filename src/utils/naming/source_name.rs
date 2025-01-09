use crate::utils::*;

pub struct SourceName;

impl SourceName {
    #[must_use]
    pub fn get(metadata: &Metadata) -> String {
        let name = Self::get_unsanitized(metadata);
        Sanitizer::execute(name)
    }

    #[must_use]
    pub fn get_unsanitized(metadata: &Metadata) -> String {
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
