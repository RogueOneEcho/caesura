use crate::formats::TargetFormat;
use crate::formats::TargetFormat::*;
use crate::naming::{SourceName, FORBIDDEN_CHARACTERS};
use crate::source::Source;

pub struct TargetName;

impl TargetName {
    #[must_use]
    pub fn get(source: &Source, target: &TargetFormat) -> String {
        let prefix = SourceName::get(source);
        let format = match target {
            Flac => "FLAC",
            _320 => "320",
            V0 => "V0",
        };
        let media = source.metadata.media.clone();
        let name = format!("{prefix} [{media} {format}]");
        name.replace(&FORBIDDEN_CHARACTERS[..], "_")
    }
}
