use std::time::Duration;

use insta::assert_snapshot;

use crate::commands::inspect::format::DisableStyleGuard;
use crate::commands::inspect::track_info::TrackInfo;

/// Test that the properties table renders mixed formats, sample rates, and bit depths.
#[test]
fn properties_table_mixed_formats() {
    // Arrange
    let _guard = DisableStyleGuard::new();
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
    let output = TrackInfo::format_properties_table(&tracks);

    // Assert
    assert_snapshot!(output);
}
