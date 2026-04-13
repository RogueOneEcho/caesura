use std::process::Stdio;

use crate::prelude::*;
use crate::testing_prelude::*;
use tokio::join;

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
    let flac_path = FlacGenerator::new()
        .with_artist("Test Artist")
        .with_album("Test Album")
        .with_title("Test Track")
        .with_track_number("1")
        .with_date("2000")
        .with_vorbis_tag("RELEASEDATE", "1st January 2001")
        .with_duration_secs(3)
        .generate(&temp)
        .await?;
    let mp3_path = temp.join("01 - Test Track.mp3");
    let source_dir = flac_path.parent().expect("flac should have parent");
    let flac = FlacFile::new(flac_path.clone(), &source_dir.to_path_buf());
    let tags = flac.id3_tags()?.clone();
    let decode = CommandInfo {
        program: FLAC.to_owned(),
        args: vec![
            "-dcs".to_owned(),
            "--".to_owned(),
            flac_path.to_string_lossy().to_string(),
        ],
    };
    let encode = CommandInfo {
        program: LAME.to_owned(),
        args: vec![
            "-S".to_owned(),
            "-h".to_owned(),
            "-b".to_owned(),
            "320".to_owned(),
            "--ignore-tag-errors".to_owned(),
            "-".to_owned(),
            mp3_path.to_string_lossy().to_string(),
        ],
    };
    let mut decode_cmd = decode
        .to_command()
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let pipe: Stdio = decode_cmd
        .stdout
        .take()
        .expect("should take stdout")
        .try_into()
        .expect("should convert to pipe");
    let encode_cmd = encode
        .to_command()
        .stdin(pipe)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let (decode_result, encode_result) = join!(decode_cmd.wait(), encode_cmd.wait_with_output());
    decode_result?;
    let encode_output = encode_result?;
    assert!(encode_output.status.success(), "lame encoding failed");
    save_id3v2_deterministic(tags, &mp3_path)?;

    // Act
    let result = TrackInfo::read(source_dir, &mp3_path);

    // Assert
    assert!(
        result.is_ok(),
        "TrackInfo::read should tolerate non-ISO-8601 TDRL tags"
    );
    Ok(())
}
