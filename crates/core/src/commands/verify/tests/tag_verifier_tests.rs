use crate::testing_prelude::*;
use lofty::tag::{Accessor, ItemKey, ItemValue, Tag, TagItem, TagType};

fn full_tags() -> Tag {
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_artist("Artist".into());
    tag.set_album("Album".into());
    tag.set_title("Title".into());
    tag.set_track(1);
    tag.set_disk(1);
    tag.push(TagItem::new(
        ItemKey::Composer,
        ItemValue::Text("Composer".into()),
    ));
    tag
}

fn minimal_tags() -> Tag {
    Tag::new(TagType::Id3v2)
}

fn classical_source() -> Source {
    let mut source = Source::mock();
    source.group.tags = vec!["classical".to_owned()];
    source.group.music_info = Some(Credits {
        composers: vec![Credit {
            id: 12345,
            name: "Test Composer".to_owned(),
        }],
        ..Credits::default()
    });
    source
}

fn mock_flac(is_multi_disc: bool) -> FlacFile {
    let source_dir = PathBuf::from("/tmp");
    let mut flac = FlacFile::new(PathBuf::from("/tmp/test.flac"), &source_dir);
    if is_multi_disc {
        flac.disc_context = Some(DiscContext {
            track_padding: 2,
            is_multi_disc: true,
            disc_count: 2,
        });
    }
    flac
}

#[test]
fn check_artist_tag_present() {
    let tags = full_tags();
    assert_eq!(check_artist_tag(&tags), None);
}

#[test]
fn check_artist_tag_missing() {
    let tags = minimal_tags();
    assert_eq!(check_artist_tag(&tags), Some("artist".to_owned()));
}

#[test]
fn check_album_tag_present() {
    let tags = full_tags();
    assert_eq!(check_album_tag(&tags), None);
}

#[test]
fn check_album_tag_missing() {
    let tags = minimal_tags();
    assert_eq!(check_album_tag(&tags), Some("album".to_owned()));
}

#[test]
fn check_title_tag_present() {
    let tags = full_tags();
    assert_eq!(check_title_tag(&tags), None);
}

#[test]
fn check_title_tag_missing() {
    let tags = minimal_tags();
    assert_eq!(check_title_tag(&tags), Some("title".to_owned()));
}

#[test]
fn check_composer_tag_classical_present() {
    let tags = full_tags();
    let source = classical_source();
    assert_eq!(check_composer_tag(&tags, &source), None);
}

#[test]
fn check_composer_tag_classical_missing() {
    let tags = minimal_tags();
    let source = classical_source();
    assert_eq!(
        check_composer_tag(&tags, &source),
        Some("composer".to_owned())
    );
}

#[test]
fn check_composer_tag_not_classical() {
    let tags = minimal_tags();
    let source = Source::mock();
    assert_eq!(check_composer_tag(&tags, &source), None);
}

#[test]
fn check_composer_tag_classical_no_credited_composers() {
    let tags = minimal_tags();
    let mut source = Source::mock();
    source.group.tags = vec!["classical".to_owned()];
    assert_eq!(check_composer_tag(&tags, &source), None);
}

#[test]
fn check_track_number_tag_present() {
    let tags = full_tags();
    assert_eq!(check_track_number_tag(&tags), None);
}

#[test]
fn check_track_number_tag_missing() {
    let tags = minimal_tags();
    assert_eq!(
        check_track_number_tag(&tags),
        Some("track_number".to_owned())
    );
}

#[test]
fn check_disc_number_tag_multi_disc_present() {
    let tags = full_tags();
    let flac = mock_flac(true);
    assert_eq!(check_disc_number_tag(&tags, &flac), None);
}

#[test]
fn check_disc_number_tag_multi_disc_missing() {
    let tags = minimal_tags();
    let flac = mock_flac(true);
    assert_eq!(
        check_disc_number_tag(&tags, &flac),
        Some("disc_number".to_owned())
    );
}

#[test]
fn check_disc_number_tag_single_disc() {
    let tags = minimal_tags();
    let flac = mock_flac(false);
    assert_eq!(check_disc_number_tag(&tags, &flac), None);
}

#[tokio::test]
async fn tag_verifier_execute_no_vorbis_block() {
    // Arrange
    let dir = TempDirectory::create("tag_verifier_execute_no_vorbis_block");
    let flac_path = FlacGenerator::new()
        .with_filename("track.flac")
        .omit_vorbis_comments()
        .generate(&dir)
        .await
        .expect("generate should succeed");
    let flac = FlacFile::new(flac_path.clone(), &dir.to_path_buf());
    let source = Source::mock();

    // Act
    let output = TagVerifier::execute(&flac, &source).expect("should not fail hard");

    // Assert
    assert_eq!(output, vec![SourceIssue::NoTags { path: flac_path }]);
}

/// A non-numeric track number like "bonus" is present as a string
/// and should not be flagged as missing.
#[test]
fn check_track_number_tag_non_numeric_string() {
    let mut tag = Tag::new(TagType::VorbisComments);
    tag.push(TagItem::new(
        ItemKey::TrackNumber,
        ItemValue::Text("bonus".into()),
    ));
    assert_eq!(check_track_number_tag(&tag), None);
}

/// A non-parseable TRACKNUMBER is valid Vorbis but not convertible to
/// `ID3v2`, producing `InvalidTags` instead of `MissingTags`.
#[tokio::test]
async fn tag_verifier_execute_non_numeric_track() {
    let dir = TempDirectory::create("tag_verifier_non_numeric_track");
    let flac_path = FlacGenerator::new()
        .with_filename("track.flac")
        .with_artist("Test Artist")
        .with_title("Test Title")
        .with_album("Test Album")
        .with_vorbis_tag("TRACKNUMBER", "bonus")
        .generate(&dir)
        .await
        .expect("generate should succeed");
    let flac = FlacFile::new(flac_path.clone(), &dir.to_path_buf());
    let source = Source::mock();

    // Act
    let output = TagVerifier::execute(&flac, &source).expect("should not fail hard");

    // Assert
    assert_eq!(
        output,
        vec![SourceIssue::InvalidTags {
            path: flac_path,
            tags: vec!["track_number".to_owned()],
        }]
    );
}

/// A track with no TRACKNUMBER at all produces `MissingTags`.
#[tokio::test]
async fn tag_verifier_execute_missing_track() {
    let dir = TempDirectory::create("tag_verifier_missing_track");
    let flac_path = FlacGenerator::new()
        .with_filename("track.flac")
        .with_artist("Test Artist")
        .with_title("Test Title")
        .with_album("Test Album")
        .generate(&dir)
        .await
        .expect("generate should succeed");
    let flac = FlacFile::new(flac_path.clone(), &dir.to_path_buf());
    let source = Source::mock();

    // Act
    let output = TagVerifier::execute(&flac, &source).expect("should not fail hard");

    // Assert
    assert_eq!(
        output,
        vec![SourceIssue::MissingTags {
            path: flac_path,
            tags: vec!["track_number".to_owned()],
        }]
    );
}

#[test]
fn invalid_tags_not_reportable() {
    let issue = SourceIssue::InvalidTags {
        path: PathBuf::from("/a.flac"),
        tags: vec!["track_number".to_owned()],
    };
    assert!(!issue.is_reportable());
}
