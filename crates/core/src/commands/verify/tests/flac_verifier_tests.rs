use crate::testing_prelude::*;

#[test]
fn check_subdirectory_variants() {
    let source_dir = PathBuf::from("source/dir");

    // Good source: all flacs share the root, no extra subdirectory.
    let output = check_subdirectory(&[
        FlacFile::new(PathBuf::from("source/dir/a.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/b.flac"), &source_dir),
    ]);
    assert_eq!(output, None);

    // Bad source: all flacs share the 'a' subdirectory.
    let output = check_subdirectory(&[
        FlacFile::new(PathBuf::from("source/dir/a/b.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/a/c.flac"), &source_dir),
    ]);
    assert_eq!(
        output,
        Some(SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("a")
        })
    );

    // Good multi-disc source: discs separate items by subdirectory.
    let output = check_subdirectory(&[
        FlacFile::new(PathBuf::from("source/dir/CD1/a.flac"), &source_dir),
        FlacFile::new(PathBuf::from("source/dir/CD2/b.flac"), &source_dir),
    ]);
    assert_eq!(output, None);

    // Bad single-file source: the sole flac sits in an unnecessary subdirectory.
    let output = check_subdirectory(&[FlacFile::new(
        PathBuf::from("source/dir/c/d.flac"),
        &source_dir,
    )]);
    assert_eq!(
        output,
        Some(SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("c")
        })
    );

    // Good single-file release directly in the source directory.
    let output = check_subdirectory(&[FlacFile::new(
        PathBuf::from("/root/album/track.flac"),
        &PathBuf::from("/root/album/"),
    )]);
    assert_eq!(output, None);
}

#[test]
fn check_flac_count_matches() {
    let source = Source::mock();
    let actual = source.torrent.get_flacs().len();
    assert_eq!(check_flac_count(&source, actual), None);
}

#[test]
fn check_flac_count_mismatch() {
    let source = Source::mock();
    assert_eq!(
        check_flac_count(&source, 5),
        Some(SourceIssue::FlacCount {
            expected: 1,
            actual: 5
        })
    );
}

#[test]
fn check_path_length_within_limit() {
    let path = PathBuf::from("a".repeat(180));
    assert_eq!(check_path_length(&path), None);
}

#[test]
fn check_path_length_exceeds_limit() {
    let path = PathBuf::from("a".repeat(185));
    assert_eq!(
        check_path_length(&path),
        Some(SourceIssue::Length {
            path: path.clone(),
            excess: 5,
        })
    );
}
