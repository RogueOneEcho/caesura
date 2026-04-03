use crate::prelude::*;

/// Generate directory names for transcodes.
pub struct TranscodeName;

impl TranscodeName {
    /// Sanitized directory name for a transcode.
    #[must_use]
    pub fn get(metadata: &Metadata, target: TargetFormat) -> String {
        let prefix = SourceName::get(metadata);
        let format = target.get_name();
        let media = metadata.media.to_string();
        let name = format!("{prefix} [{media} {format}]");
        Sanitizer::name().execute(name).output
    }
}
