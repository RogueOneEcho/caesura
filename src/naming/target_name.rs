use crate::formats::TargetFormat;
use crate::naming::{Sanitizer, SourceName};
use crate::source::Metadata;

pub struct TargetName;

impl TargetName {
    #[must_use]
    pub fn get(metadata: &Metadata, target: &TargetFormat) -> String {
        let prefix = SourceName::get(metadata);
        let format = target.get_name();
        let media = metadata.media.clone();
        let name = format!("{prefix} [{media} {format}]");
        Sanitizer::execute(name)
    }
}
