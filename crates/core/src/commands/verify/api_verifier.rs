use crate::prelude::*;

/// Check the source group category is "Music".
pub(crate) fn check_category(source: &Source) -> Option<SourceIssue> {
    if source.group.category_name != "Music" {
        return Some(SourceIssue::Category {
            actual: source.group.category_name.clone(),
        });
    }
    None
}

/// Check the source is not a scene release.
pub(crate) fn check_scene(source: &Source) -> Option<SourceIssue> {
    if source.torrent.scene {
        return Some(SourceIssue::Scene);
    }
    None
}

/// Check the source does not look like an unmarked scene release.
///
/// Scene releases typically use underscores instead of spaces in both the
/// directory name and file names.
pub(crate) fn check_possible_scene(source: &Source) -> Option<SourceIssue> {
    if !source.torrent.file_path.contains(' ') && !source.torrent.file_list.contains(' ') {
        return Some(SourceIssue::PossibleScene);
    }
    None
}

/// Check the source does not have lossy master approval.
pub(crate) fn check_lossy_master(source: &Source) -> Option<SourceIssue> {
    if source.torrent.lossy_master_approved == Some(true) {
        return Some(SourceIssue::LossyMaster);
    }
    None
}

/// Check the source does not have lossy web approval.
pub(crate) fn check_lossy_web(source: &Source) -> Option<SourceIssue> {
    if source.torrent.lossy_web_approved == Some(true) {
        return Some(SourceIssue::LossyWeb);
    }
    None
}

/// Check the source is not trumpable.
pub(crate) fn check_trumpable(source: &Source) -> Option<SourceIssue> {
    if source.torrent.trumpable == Some(true) {
        return Some(SourceIssue::Trumpable);
    }
    None
}

/// Check the source edition is confirmed.
pub(crate) fn check_unconfirmed(source: &Source) -> Option<SourceIssue> {
    if source.torrent.remastered == Some(false) {
        return Some(SourceIssue::Unconfirmed);
    }
    None
}

/// Check the source does not have any excluded tags.
pub(crate) fn check_excluded_tags(source: &Source, exclude_tags: &[String]) -> Option<SourceIssue> {
    let excluded: Vec<String> = exclude_tags
        .iter()
        .filter(|x| source.group.tags.contains(x))
        .cloned()
        .collect();
    if !excluded.is_empty() {
        return Some(SourceIssue::Excluded { tags: excluded });
    }
    None
}

/// Check there are target formats available for transcoding.
pub(crate) fn check_existing_formats(
    source: &Source,
    target_formats: &BTreeSet<TargetFormat>,
) -> Option<SourceIssue> {
    if target_formats.is_empty() {
        return Some(SourceIssue::Existing {
            formats: source.existing.clone(),
        });
    }
    None
}
