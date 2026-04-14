use crate::testing_prelude::*;

#[test]
fn check_category_music() {
    let source = Source::mock();
    assert_eq!(check_category(&source), None);
}

#[test]
fn check_category_not_music() {
    let mut source = Source::mock();
    source.group.category_name = "Applications".to_owned();
    assert_eq!(
        check_category(&source),
        Some(SourceIssue::Category {
            actual: "Applications".to_owned()
        })
    );
}

#[test]
fn check_scene_not_scene() {
    let source = Source::mock();
    assert_eq!(check_scene(&source), None);
}

#[test]
fn check_scene_is_scene() {
    let mut source = Source::mock();
    source.torrent.scene = true;
    assert_eq!(check_scene(&source), Some(SourceIssue::Scene));
}

#[test]
fn check_lossy_master_not_approved() {
    let source = Source::mock();
    assert_eq!(check_lossy_master(&source), None);
}

#[test]
fn check_lossy_master_approved() {
    let mut source = Source::mock();
    source.torrent.lossy_master_approved = Some(true);
    assert_eq!(check_lossy_master(&source), Some(SourceIssue::LossyMaster));
}

#[test]
fn check_lossy_web_not_approved() {
    let source = Source::mock();
    assert_eq!(check_lossy_web(&source), None);
}

#[test]
fn check_lossy_web_approved() {
    let mut source = Source::mock();
    source.torrent.lossy_web_approved = Some(true);
    assert_eq!(check_lossy_web(&source), Some(SourceIssue::LossyWeb));
}

#[test]
fn check_trumpable_not_trumpable() {
    let source = Source::mock();
    assert_eq!(check_trumpable(&source), None);
}

#[test]
fn check_trumpable_is_trumpable() {
    let mut source = Source::mock();
    source.torrent.trumpable = Some(true);
    assert_eq!(check_trumpable(&source), Some(SourceIssue::Trumpable));
}

#[test]
fn check_unconfirmed_confirmed() {
    let source = Source::mock();
    assert_eq!(check_unconfirmed(&source), None);
}

#[test]
fn check_unconfirmed_unconfirmed() {
    let mut source = Source::mock();
    source.torrent.remastered = Some(false);
    assert_eq!(check_unconfirmed(&source), Some(SourceIssue::Unconfirmed));
}

#[test]
fn check_excluded_tags_no_match() {
    let source = Source::mock();
    let exclude = vec!["hip.hop".to_owned()];
    assert_eq!(check_excluded_tags(&source, &exclude), None);
}

#[test]
fn check_excluded_tags_match() {
    let source = Source::mock();
    let exclude = vec!["rock".to_owned()];
    assert_eq!(
        check_excluded_tags(&source, &exclude),
        Some(SourceIssue::Excluded {
            tags: vec!["rock".to_owned()]
        })
    );
}

#[test]
fn check_targets_available() {
    let source = Source::mock();
    let configured = TargetFormat::all();
    assert_eq!(check_targets(&source, &configured), None);
}

#[test]
fn check_targets_none_available() {
    let mut source = Source::mock();
    source.targets = BTreeSet::new();
    let configured = TargetFormat::all();
    assert_eq!(
        check_targets(&source, &configured),
        Some(SourceIssue::NoTargets {
            formats: TargetFormat::all()
        })
    );
}

#[test]
fn check_possible_scene_has_spaces() {
    let source = Source::mock();
    assert_eq!(check_possible_scene(&source), None);
}

#[test]
fn check_possible_scene_no_spaces() {
    let mut source = Source::mock();
    source.torrent.file_path = "Artist-Album-24BIT-WEB-FLAC-2026-GRP".to_owned();
    source.torrent.file_list =
        "01_Track_One.flac{{{10000}}}|||02_Track_Two.flac{{{10000}}}".to_owned();
    assert_eq!(
        check_possible_scene(&source),
        Some(SourceIssue::PossibleScene)
    );
}

#[test]
/// File path without spaces but file list with spaces should pass.
fn check_possible_scene_file_path_no_spaces() {
    let mut source = Source::mock();
    source.torrent.file_path = "Artist-Album-2026".to_owned();
    source.torrent.file_list =
        "01 Track One.flac{{{10000}}}|||02 Track Two.flac{{{10000}}}".to_owned();
    assert_eq!(check_possible_scene(&source), None);
}
