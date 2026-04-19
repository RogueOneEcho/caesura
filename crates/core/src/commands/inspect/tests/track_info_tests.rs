use crate::prelude::*;
use crate::testing_prelude::*;
use lofty::config::ParsingMode;

/// Tolerate non-ISO-8601 values in `ID3v2` TDRL timestamp frames.
///
/// - A freeform Vorbis `RELEASEDATE` comment is converted to a plain text
///   TDRL frame during transcode, but lofty parses TDRL as a timestamp frame
/// - Without relaxed parsing, this causes [`TrackInfo::read`] to fail entirely
#[tokio::test]
async fn track_info_read_mp3_with_non_iso8601_tdrl() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let temp = TempDirectory::create("non_iso8601_tdrl");
    let flac_path = FlacGenerator::mock()
        .with_vorbis_tag("RELEASEDATE", "1st January 2001")
        .generate(&temp)
        .await?;
    let mp3_path = temp.join("01 - Test Track.mp3");
    let source_dir = temp.to_path_buf();
    let flac = FlacFile::new(flac_path, &source_dir);
    transcode_to_mp3(&flac, &mp3_path, TargetFormat::_320).await?;

    // Act
    let info = TrackInfo::read(&source_dir, &mp3_path)?;

    // Assert
    assert!(has_native_tag(&info, "TPE1"));
    assert!(has_native_tag(&info, "TIT2"));
    assert!(has_native_tag(&info, "TRCK"));
    assert!(has_native_tag(&info, "TDRC"));
    assert!(!has_native_tag(&info, "TDRL"));
    assert_eq!(info.parsing_mode, Some(ParsingMode::Relaxed));
    assert_eq!(
        info.parsing_error.as_deref(),
        Some("Encountered an invalid timestamp: Timestamp segment contains non-digit characters"),
    );
    Ok(())
}

/// Tolerate dot-separated dates in `ID3v2` TDRC timestamp frames.
///
/// - Asian distributors (Genie, etc.) use `DATE=2025.11.20` in Vorbis comments
/// - LAME copies this verbatim into the TDRC frame during transcode
/// - Lofty only recognizes `-`, `T`, `:` as timestamp separators
/// - Without relaxed parsing, `.` causes `TrackInfo::read` to fail
/// - See Serial-ATA/lofty-rs#647
#[tokio::test]
async fn track_info_read_mp3_with_dot_separated_tdrc() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let temp = TempDirectory::create("dot_separated_tdrc");
    let flac_path = FlacGenerator::mock()
        .with_date("2025.11.20")
        .generate(&temp)
        .await?;
    let mp3_path = temp.join("01 - Test Track.mp3");
    let source_dir = temp.to_path_buf();
    let flac = FlacFile::new(flac_path, &source_dir);
    transcode_to_mp3(&flac, &mp3_path, TargetFormat::_320).await?;

    // Act
    let info = TrackInfo::read(&source_dir, &mp3_path)?;

    // Assert
    assert!(has_native_tag(&info, "TPE1"));
    assert!(has_native_tag(&info, "TIT2"));
    assert!(has_native_tag(&info, "TRCK"));
    assert!(!has_native_tag(&info, "TDRC"));
    assert_eq!(info.parsing_mode, Some(ParsingMode::Relaxed));
    assert_eq!(
        info.parsing_error.as_deref(),
        Some("Encountered an invalid timestamp: Timestamp segment contains non-digit characters"),
    );
    Ok(())
}

/// Transcode a single FLAC to MP3 using [`TranscodeJob`].
async fn transcode_to_mp3(
    flac: &FlacFile,
    output: &Path,
    format: TargetFormat,
) -> Result<(), Failure<TranscodeAction>> {
    let sox = Ref::new(SoxFactory::new(Ref::new(SoxOptions {
        sox_path: None,
        sox_ng: false,
    })));
    let job = TranscodeJob {
        id: "test".to_owned(),
        variant: Variant::Transcode(
            Decode {
                input: flac.path.clone(),
                resample_rate: None,
                repeatable: true,
                sox,
            },
            Encode {
                output: output.to_path_buf(),
                format,
            },
        ),
        tags: Some(flac.id3_tags()?.clone()),
        exclude_vorbis_comments: Vec::new(),
    };
    job.execute().await
}

fn has_native_tag(info: &TrackInfo, native_key: &str) -> bool {
    info.tags
        .iter()
        .any(|t| t.native.as_deref() == Some(native_key))
}
