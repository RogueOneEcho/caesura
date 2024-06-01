use crate::errors::AppError;
use crate::fs::FlacFile;
use crate::verify::SourceRule;
use crate::verify::SourceRule::{NoAlbumTag, NoArtistTag, NoTitleTag, NoTrackNumberTag};
use log::warn;

pub struct TagVerifier;

impl TagVerifier {
    pub fn execute(flac: &FlacFile, media: &str) -> Result<Vec<SourceRule>, AppError> {
        let tags = flac.get_tags()?;
        let mut errors: Vec<SourceRule> = Vec::new();
        if tags.artist().is_none() {
            errors.push(NoArtistTag(flac.file_name.clone()));
        }
        if tags.album().is_none() {
            errors.push(NoAlbumTag(flac.file_name.clone()));
        }
        if tags.title().is_none() {
            errors.push(NoTitleTag(flac.file_name.clone()));
        }
        if tags.track_number().is_none() {
            // TODO MUST confirm vinyl media comparison works
            if media.to_uppercase() == "vinyl" {
                warn!("Unable to verify if the track number is valid. Vinyl releases can have non-standard track numbers (e.g. A1, A2, etc).");
            } else {
                errors.push(NoTrackNumberTag(flac.file_name.clone()));
            }
        }
        Ok(errors)
    }
}
