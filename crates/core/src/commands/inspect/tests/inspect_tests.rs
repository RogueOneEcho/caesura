use crate::testing_prelude::*;
use lofty::tag::ItemKey;

/// Test that [`InspectFactory::create`] returns FLAC metadata for a multi-disc source directory.
#[tokio::test]
async fn inspect_factory_flac() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let path = SAMPLE_SOURCES_DIR.join(config.dir_name());
    let factory = InspectFactory::new(false);

    // Act
    let output = factory.create(&path)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that [`InspectFactory::create`] returns MP3 metadata for a 320kbps transcode.
#[tokio::test]
async fn inspect_factory_320() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::_320).await;
    let path = transcode.transcode_dir();
    let factory = InspectFactory::new(false);

    // Act
    let output = factory.create(&path)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that [`InspectFactory::create`] returns MP3 metadata for a V0 transcode.
#[tokio::test]
async fn inspect_factory_v0() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::V0).await;
    let path = transcode.transcode_dir();
    let factory = InspectFactory::new(false);

    // Act
    let output = factory.create(&path)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that [`InspectFactory::create`] formats a 48 kHz sample rate without a fractional part.
#[tokio::test]
async fn inspect_factory_flac_48khz() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get(SampleFormat::FLAC16_48).await;
    let path = SAMPLE_SOURCES_DIR.join(config.dir_name());
    let factory = InspectFactory::new(false);

    // Act
    let output = factory.create(&path)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that [`InspectFactory::create`] handles mixed FLAC and MP3 files in the same directory.
#[tokio::test]
#[expect(
    clippy::indexing_slicing,
    reason = "test with controlled config guarantees indices exist"
)]
async fn inspect_factory_mixed() -> Result<(), TestError> {
    // Arrange
    let config = AlbumProvider::get_advanced(AlbumConfig::multi_disc()).await;
    let flac_dir = SAMPLE_SOURCES_DIR.join(config.dir_name());
    let transcode = TranscodeProvider::get_advanced(&config, TargetFormat::_320).await;
    let mp3_dir = transcode.transcode_dir();

    let temp = TempDirectory::create("mixed_format");
    copy(
        flac_dir
            .join("Disc 1")
            .join(config.track_filename(&config.tracks[0])),
        temp.join(config.track_filename(&config.tracks[0])),
    )?;
    let mp3_filename = config
        .track_filename(&config.tracks[1])
        .replace(".flac", ".mp3");
    copy(
        mp3_dir.join("Disc 1").join(&mp3_filename),
        temp.join(&mp3_filename),
    )?;
    let factory = InspectFactory::new(false);

    // Act
    let output = factory.create(&temp)?;

    // Assert
    assert_inspect_snapshot!(output);
    Ok(())
}

/// Test that the properties table renders mixed formats, sample rates, and bit depths.
#[test]
fn inspect_factory_properties_table_mixed_formats() {
    // Arrange
    let factory = InspectFactory::new(false);
    let tracks = vec![
        TrackInfo {
            track: Some("1".to_owned()),
            duration: Duration::from_secs(312),
            file_size: 30_408_704,
            bit_rate: 780,
            ..TrackInfo::mock_flac()
        },
        TrackInfo {
            track: Some("2".to_owned()),
            duration: Duration::from_secs(245),
            file_size: 47_710_208,
            bit_rate: 1558,
            sample_rate: 96000,
            bit_depth: Some(24),
            ..TrackInfo::mock_flac()
        },
        TrackInfo {
            track: Some("3".to_owned()),
            duration: Duration::from_secs(198),
            file_size: 19_922_944,
            bit_rate: 805,
            sample_rate: 48000,
            bit_depth: Some(24),
            ..TrackInfo::mock_flac()
        },
        TrackInfo {
            track: Some("4".to_owned()),
            duration: Duration::from_secs(312),
            file_size: 12_288_000,
            ..TrackInfo::mock_mp3()
        },
        TrackInfo {
            track: Some("5".to_owned()),
            duration: Duration::from_secs(312),
            file_size: 9_523_200,
            bit_rate: 245,
            sample_rate: 48000,
            ..TrackInfo::mock_mp3()
        },
    ];

    // Act
    let output = factory.format_properties_table(&tracks);

    // Assert
    assert_snapshot!(output);
}

/// Test that the tags table wraps a long single-line value onto multiple visual rows.
#[test]
fn format_tags_table_with_long_value() {
    // Arrange
    let factory = InspectFactory::new(false);
    let track = TrackInfo {
        tags: vec![TagEntry {
            key: ItemKey::CopyrightMessage,
            native: Some("COPYRIGHT".to_owned()),
            value: "(c) 2011 Long artist name via Long record label name (c) 1981 Long artist name via Long record label name".to_owned(),
        }],
        ..TrackInfo::mock_flac()
    };

    // Act
    let output = factory.format_all_tags(&[track]);

    // Assert
    assert_snapshot!(output);
}

/// Test that the tags table truncates a multi-line value at 3 visual rows.
#[test]
fn format_tags_table_with_multiline_value() {
    // Arrange
    let factory = InspectFactory::new(false);
    let lyrics = (1..=20)
        .map(|i| format!("Lyric line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let track = TrackInfo {
        tags: vec![TagEntry {
            key: ItemKey::Lyrics,
            native: Some("LYRICS".to_owned()),
            value: lyrics,
        }],
        ..TrackInfo::mock_flac()
    };

    // Act
    let output = factory.format_all_tags(&[track]);

    // Assert
    assert_snapshot!(output);
}
