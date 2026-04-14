use crate::testing_prelude::*;

fn mock_source() -> Source {
    Source {
        torrent: gazelle_api::Torrent::mock(),
        group: gazelle_api::Group::mock(),
        existing: BTreeSet::new(),
        format: SourceFormat::Flac,
        directory: PathBuf::from("/tmp/test"),
        metadata: Metadata::new(&gazelle_api::Group::mock(), &gazelle_api::Torrent::mock()),
        url: get_permalink(&RED_URL.to_owned(), 123, 456),
    }
}

#[test]
fn check_category_music() {
    let source = mock_source();
    assert_eq!(check_category(&source), None);
}

#[test]
fn check_category_not_music() {
    let mut source = mock_source();
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
    let source = mock_source();
    assert_eq!(check_scene(&source), None);
}

#[test]
fn check_scene_is_scene() {
    let mut source = mock_source();
    source.torrent.scene = true;
    assert_eq!(check_scene(&source), Some(SourceIssue::Scene));
}

#[test]
fn check_lossy_master_not_approved() {
    let source = mock_source();
    assert_eq!(check_lossy_master(&source), None);
}

#[test]
fn check_lossy_master_approved() {
    let mut source = mock_source();
    source.torrent.lossy_master_approved = Some(true);
    assert_eq!(check_lossy_master(&source), Some(SourceIssue::LossyMaster));
}

#[test]
fn check_lossy_web_not_approved() {
    let source = mock_source();
    assert_eq!(check_lossy_web(&source), None);
}

#[test]
fn check_lossy_web_approved() {
    let mut source = mock_source();
    source.torrent.lossy_web_approved = Some(true);
    assert_eq!(check_lossy_web(&source), Some(SourceIssue::LossyWeb));
}

#[test]
fn check_trumpable_not_trumpable() {
    let source = mock_source();
    assert_eq!(check_trumpable(&source), None);
}

#[test]
fn check_trumpable_is_trumpable() {
    let mut source = mock_source();
    source.torrent.trumpable = Some(true);
    assert_eq!(check_trumpable(&source), Some(SourceIssue::Trumpable));
}

#[test]
fn check_unconfirmed_confirmed() {
    let source = mock_source();
    assert_eq!(check_unconfirmed(&source), None);
}

#[test]
fn check_unconfirmed_unconfirmed() {
    let mut source = mock_source();
    source.torrent.remastered = Some(false);
    assert_eq!(check_unconfirmed(&source), Some(SourceIssue::Unconfirmed));
}

#[test]
fn check_excluded_tags_no_match() {
    let source = mock_source();
    let exclude = vec!["hip.hop".to_owned()];
    assert_eq!(check_excluded_tags(&source, &exclude), None);
}

#[test]
fn check_excluded_tags_match() {
    let source = mock_source();
    let exclude = vec!["rock".to_owned()];
    assert_eq!(
        check_excluded_tags(&source, &exclude),
        Some(SourceIssue::Excluded {
            tags: vec!["rock".to_owned()]
        })
    );
}

#[test]
fn check_existing_formats_available() {
    let source = mock_source();
    let targets = BTreeSet::from([TargetFormat::_320]);
    assert_eq!(check_existing_formats(&source, &targets), None);
}

#[test]
fn check_existing_formats_none_available() {
    let mut source = mock_source();
    source.existing = BTreeSet::from([ExistingFormat::_320, ExistingFormat::V0]);
    let targets = BTreeSet::new();
    assert_eq!(
        check_existing_formats(&source, &targets),
        Some(SourceIssue::Existing {
            formats: BTreeSet::from([ExistingFormat::_320, ExistingFormat::V0])
        })
    );
}

#[test]
fn check_possible_scene_has_spaces() {
    let source = mock_source();
    assert_eq!(check_possible_scene(&source), None);
}

#[test]
fn check_possible_scene_no_spaces() {
    let mut source = mock_source();
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
    let mut source = mock_source();
    source.torrent.file_path = "Artist-Album-2026".to_owned();
    source.torrent.file_list =
        "01 Track One.flac{{{10000}}}|||02 Track Two.flac{{{10000}}}".to_owned();
    assert_eq!(check_possible_scene(&source), None);
}
