use crate::prelude::*;

/// Generate directory names for spectrograms.
#[injectable]
pub struct SpectrogramName{
    source_name: Ref<SourceName>
}

impl SpectrogramName {
    /// Sanitized directory name for spectrograms.
    #[must_use]
    pub fn get(&self, metadata: &Metadata) -> String {
        let prefix = self.source_name.get(metadata);
        let media = metadata.media.clone();
        let name = format!("{prefix} [{media} SPECTROGRAMS]");
        Sanitizer::execute(name)
    }
}
