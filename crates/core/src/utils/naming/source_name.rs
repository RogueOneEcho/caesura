use crate::prelude::*;

/// Generate directory names for sources.
#[injectable]
pub struct SourceName{
    shared_options: Ref<SharedOptions>,
}

impl SourceName {
    /// Sanitized directory name for a source.
    #[must_use]
    pub fn get(&self, metadata: &Metadata) -> String {
        let name = self.get_unsanitized(metadata);
        Sanitizer::execute(name)
    }

    /// Directory name for a source without sanitization.
    #[must_use]
    pub fn get_unsanitized(&self, metadata: &Metadata) -> String {
        if metadata.remaster_title.is_empty() {
            self.shared_options
                .transcoded_name_template_fallback
                .replace("{artist}", &metadata.artist)
                .replace("{album}", &metadata.album)
                .replace("{year}", &metadata.year.to_string())
        } else {
            self.shared_options
                .transcoded_name_template
                .replace("{artist}", &metadata.artist)
                .replace("{album}", &metadata.album)
                .replace("{remaster_title}", &metadata.remaster_title)
                .replace("{year}", &metadata.year.to_string())
        }
    }
}
