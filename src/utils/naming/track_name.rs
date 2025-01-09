use crate::utils::*;

use lofty::prelude::Accessor;

pub struct TrackName;

impl TrackName {
    #[must_use]
    pub fn get(flac: &FlacFile) -> Option<String> {
        let tags = get_vorbis_tags(flac).ok()?;
        let track_number = tags.track()?;
        let title = tags.title()?;
        let file_name = format!("{track_number:0>2} {title}");
        Some(Sanitizer::execute(file_name))
    }
}
