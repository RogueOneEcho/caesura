use crate::prelude::*;
use lofty::prelude::Accessor;
use lofty::prelude::ItemKey::Composer;
use lofty::tag::Tag;

/// Verify FLAC files have required metadata tags.
pub(crate) struct TagVerifier;

impl TagVerifier {
    /// Verify required tags on a FLAC file and return any missing tag issue.
    pub(crate) fn execute(
        flac: &FlacFile,
        source: &Source,
    ) -> Result<Option<SourceIssue>, Failure<TranscodeAction>> {
        let tags = match flac.id3_tags() {
            Ok(tags) => tags,
            Err(failure) if is_no_vorbis_comments(&failure) => {
                return Ok(Some(SourceIssue::NoTags {
                    path: flac.path.clone(),
                }));
            }
            Err(failure) => return Err(failure),
        };
        let mut missing = Vec::new();
        missing.extend(check_artist_tag(tags));
        missing.extend(check_album_tag(tags));
        missing.extend(check_title_tag(tags));
        missing.extend(check_composer_tag(tags, source));
        missing.extend(check_track_number_tag(tags));
        missing.extend(check_disc_number_tag(tags, flac));
        if missing.is_empty() {
            return Ok(None);
        }
        Ok(Some(SourceIssue::MissingTags {
            path: flac.path.clone(),
            tags: missing,
        }))
    }
}

/// Check the artist tag is present.
pub(crate) fn check_artist_tag(tags: &Tag) -> Option<String> {
    if tags.artist().is_none() {
        return Some("artist".to_owned());
    }
    None
}

/// Check the album tag is present.
pub(crate) fn check_album_tag(tags: &Tag) -> Option<String> {
    if tags.album().is_none() {
        return Some("album".to_owned());
    }
    None
}

/// Check the title tag is present.
pub(crate) fn check_title_tag(tags: &Tag) -> Option<String> {
    if tags.title().is_none() {
        return Some("title".to_owned());
    }
    None
}

/// Check the composer tag is present when the source is classical.
pub(crate) fn check_composer_tag(tags: &Tag, source: &Source) -> Option<String> {
    let is_classical = source.group.tags.contains(&"classical".to_owned());
    if is_classical && tags.get(Composer).is_none() {
        return Some("composer".to_owned());
    }
    None
}

/// Check the track number tag is present.
pub(crate) fn check_track_number_tag(tags: &Tag) -> Option<String> {
    if tags.track().is_none() {
        return Some("track_number".to_owned());
    }
    None
}

/// Check the disc number tag is present when the source is multi-disc.
pub(crate) fn check_disc_number_tag(tags: &Tag, flac: &FlacFile) -> Option<String> {
    let is_multi_disc = flac
        .disc_context
        .as_ref()
        .is_some_and(|ctx| ctx.is_multi_disc);
    if is_multi_disc && tags.disk().is_none() {
        return Some("disc_number".to_owned());
    }
    None
}

/// Walk the source chain looking for [`TagsError::NoVorbisComments`].
fn is_no_vorbis_comments(failure: &Failure<TranscodeAction>) -> bool {
    let mut source: Option<&(dyn Error + 'static)> = failure.source();
    while let Some(err) = source {
        if let Some(tags_err) = err.downcast_ref::<TagsError>() {
            return matches!(tags_err, TagsError::NoVorbisComments);
        }
        source = err.source();
    }
    false
}
