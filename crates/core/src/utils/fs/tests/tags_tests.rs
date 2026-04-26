use crate::testing_prelude::*;

#[test]
fn get_numeric_from_total_format_slash_separated() {
    assert_eq!(get_numeric_from_total_format("1/1"), Some((1, 1)));
    assert_eq!(get_numeric_from_total_format("1/12"), Some((1, 12)));
    assert_eq!(get_numeric_from_total_format("2/6"), Some((2, 6)));
    assert_eq!(get_numeric_from_total_format("3/8"), Some((3, 8)));
    assert_eq!(get_numeric_from_total_format("26/9"), Some((26, 9)));
}

#[test]
fn get_numeric_from_total_format_missing_slash() {
    assert_eq!(get_numeric_from_total_format(""), None);
    assert_eq!(get_numeric_from_total_format("1"), None);
    assert_eq!(get_numeric_from_total_format("12"), None);
    assert_eq!(get_numeric_from_total_format("0"), None);
}

#[test]
fn get_numeric_from_vinyl_format_letter_prefixed() {
    assert_eq!(get_numeric_from_vinyl_format("A1"), Some((1, 1)));
    assert_eq!(get_numeric_from_vinyl_format("A"), Some((1, 1)));
    assert_eq!(get_numeric_from_vinyl_format("B"), Some((2, 1)));
    assert_eq!(get_numeric_from_vinyl_format("A12"), Some((1, 12)));
    assert_eq!(get_numeric_from_vinyl_format("B6"), Some((2, 6)));
    assert_eq!(get_numeric_from_vinyl_format("C8"), Some((3, 8)));
    assert_eq!(get_numeric_from_vinyl_format("Z9"), Some((26, 9)));
}

#[test]
fn get_numeric_from_vinyl_format_no_letter() {
    assert_eq!(get_numeric_from_vinyl_format(""), None);
    assert_eq!(get_numeric_from_vinyl_format("12"), None);
    assert_eq!(get_numeric_from_vinyl_format("1A"), None);
}

/// Regression test for lofty 0.23.3 bug where `save_to_path` left stale bytes
/// when the re-encoded metadata was smaller than the original.
///
/// - Occurs when the excluded tag is large enough that the re-encoded
///   metadata block is smaller than the original, even after lofty adds
///   a 1024-byte PADDING block to files that lack one
/// - A value over ~1040 bytes is the minimum to exceed the padding
/// - <https://github.com/Serial-ATA/lofty-rs/issues/640>
/// - <https://github.com/Serial-ATA/lofty-rs/pull/641>
#[tokio::test]
async fn exclude_vorbis_comments_from_flac_with_exclusion() -> Result<(), TestError> {
    // Arrange
    let temp = TempDirectory::create("tags_exclude_with_exclusion");
    let path = FlacGenerator::new()
        .with_duration_secs(5)
        .with_vorbis_tag("COMMENT", "X".repeat(1100))
        .generate(&temp)
        .await?;
    let keys = vec![String::from("COMMENT")];
    let size_before = file_size(&path);

    // Act
    exclude_vorbis_comments_from_flac(&path, &keys)?;

    // Assert
    let size_after = file_size(&path);
    assert!(
        size_after < size_before,
        "file should shrink after excluding tags"
    );
    assert_flac_integrity(&path).await;
    Ok(())
}

/// Verify unrecognized Vorbis comment keys are preserved after excluding tags.
#[tokio::test]
async fn exclude_vorbis_comments_from_flac_preserves_custom_tags() -> Result<(), TestError> {
    // Arrange
    let temp = TempDirectory::create("tags_preserves_custom");
    let path = FlacGenerator::new()
        .with_artist("Test Artist")
        .with_duration_secs(5)
        .with_vorbis_tag("COMMENT", "to be removed")
        .with_vorbis_tag("CUSTOM_TAG", "should survive")
        .with_vorbis_tag("SYNCEDLYRICS", "should also survive")
        .generate(&temp)
        .await?;
    let keys = vec![String::from("COMMENT")];

    // Act
    exclude_vorbis_comments_from_flac(&path, &keys)?;

    // Assert
    let output = read_vorbis_tags(&path);
    assert!(output.contains("CUSTOM_TAG=should survive"));
    assert!(output.contains("SYNCEDLYRICS=should also survive"));
    assert!(!output.contains("COMMENT="));
    Ok(())
}

/// Verify the file is not modified when no tags match the exclusion list.
#[tokio::test]
async fn exclude_vorbis_comments_from_flac_noop() -> Result<(), TestError> {
    // Arrange
    let temp = TempDirectory::create("tags_noop");
    let path = FlacGenerator::new()
        .with_artist("Test Artist")
        .with_duration_secs(5)
        .with_vorbis_tag("CUSTOM_TAG", "X".repeat(2000))
        .generate(&temp)
        .await?;
    let keys = vec![String::from("NONEXISTENT_KEY")];
    let size_before = file_size(&path);
    let hash_before = file_sha256(&path);

    // Act
    exclude_vorbis_comments_from_flac(&path, &keys)?;

    // Assert
    assert_eq!(file_size(&path), size_before, "file should not be modified");
    assert_eq!(
        file_sha256(&path),
        hash_before,
        "file content should be identical"
    );
    Ok(())
}

/// Verify that excluding `ENCODER` removes the Vorbis comment but preserves
/// the vendor string.
///
/// The FLAC Vorbis comment block has a vendor string (set by the encoder) plus
/// key-value comment pairs. These are distinct: the vendor string identifies
/// the encoder software, while an `ENCODER` comment is a user-facing tag.
/// lofty's native API keeps them separate, so removing the `ENCODER` comment
/// does not affect the vendor string. The old generic `Tag` API conflated the
/// two, making `ENCODER` unstrippable:
/// <https://github.com/Serial-ATA/lofty-rs/blob/d9eb83ba614001973f9ba3663c9f3e10dd27a702/lofty/src/flac/write.rs#L90-L104>
#[tokio::test]
async fn exclude_vorbis_comments_from_flac_encoder() -> Result<(), TestError> {
    // Arrange
    let temp = TempDirectory::create("tags_encoder");
    let path = FlacGenerator::new()
        .with_duration_secs(5)
        .with_vorbis_tag("ENCODER", "Some Encoder v1.0")
        .generate(&temp)
        .await?;
    let tags_before = read_vorbis_tags(&path);
    assert!(tags_before.contains("ENCODER=Some Encoder v1.0"));
    let keys = vec![String::from("ENCODER")];

    // Act
    exclude_vorbis_comments_from_flac(&path, &keys)?;

    // Assert
    let tags_after = read_vorbis_tags(&path);
    assert!(!tags_after.contains("ENCODER=Some Encoder v1.0"));
    assert_flac_integrity(&path).await;
    Ok(())
}

async fn assert_flac_integrity(path: &Path) {
    let result = TokioCommand::new(FLAC)
        .args(["--test", "--silent"])
        .arg(path)
        .run()
        .await;
    assert!(result.is_ok(), "flac --test failed: {}", path.display());
}

fn read_vorbis_tags(path: &Path) -> String {
    use std::process::Command;
    let output = Command::new(METAFLAC)
        .arg("--export-tags-to=-")
        .arg(path)
        .output()
        .expect("metaflac should run");
    String::from_utf8(output.stdout).expect("metaflac output should be valid UTF-8")
}

fn file_sha256(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let content = read(path).expect("file should be readable");
    format!("{:x}", Sha256::digest(&content))
}

fn file_size(path: &Path) -> u64 {
    metadata(path).expect("file should exist").len()
}
