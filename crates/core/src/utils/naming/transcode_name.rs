use crate::prelude::*;

/// Generate directory names for transcodes.
#[injectable]
pub struct TranscodeName{
    source_name: Ref<SourceName>
}

impl TranscodeName {
    /// Sanitized directory name for a transcode.
    #[must_use]
    pub fn get(&self, metadata: &Metadata, target: TargetFormat) -> String {
        let prefix = self.source_name.get(metadata);
        let format = target.get_name();
        let media = metadata.media.clone();
        let name = format!("{prefix} [{media} {format}]");
        Sanitizer::execute(name)
    }
}
