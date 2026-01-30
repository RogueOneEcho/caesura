use crate::prelude::*;
use lofty::prelude::Accessor;
use lofty::prelude::ItemKey::Composer;

/// Verify FLAC files have required metadata tags.
pub(crate) struct TagVerifier;

impl TagVerifier {
    pub(crate) fn execute(flac: &FlacFile, source: &Source) -> Result<Vec<String>, Error> {
        let tags = flac.id3_tags()?;
        let mut missing: Vec<String> = Vec::new();
        if tags.artist().is_none() {
            missing.push("artist".to_owned());
        }
        if tags.album().is_none() {
            missing.push("album".to_owned());
        }
        if tags.title().is_none() {
            missing.push("title".to_owned());
        }
        let is_classical = source.group.tags.contains(&"classical".to_owned());
        if is_classical && tags.get(&Composer).is_none() {
            missing.push("composer".to_owned());
        }
        if tags.track().is_none() {
            missing.push("track_number".to_owned());
        }
        if let Some(context) = &flac.disc_context
            && context.is_multi_disc
            && tags.disk().is_none()
        {
            missing.push("disc_number".to_owned());
        }
        Ok(missing)
    }
}
