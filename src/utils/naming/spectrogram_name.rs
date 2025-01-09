use crate::utils::*;

pub struct SpectrogramName;

impl SpectrogramName {
    #[must_use]
    pub fn get(metadata: &Metadata) -> String {
        let prefix = SourceName::get(metadata);
        let media = metadata.media.clone();
        let name = format!("{prefix} [{media} SPECTROGRAMS]");
        Sanitizer::execute(name)
    }
}
