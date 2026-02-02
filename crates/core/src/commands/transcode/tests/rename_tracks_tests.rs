use crate::testing_prelude::*;
use crate::utils::TargetFormat::{_320, V0};

/// Test `rename_tracks` with a single-disc album.
///
/// Expected output filenames: `1 Track One.mp3`, `2 Track Two.mp3`
/// (single-digit padding since max track is 2)
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn transcode_rename_tracks_single_disc() {
    let snapshot = rename_tracks_helper(AlbumConfig::single_disc()).await;
    assert_yaml_snapshot!(snapshot);
}

/// Test `rename_tracks` with a multi-disc album.
///
/// Expected output structure:
/// - `CD1/1 First Track.mp3`
/// - `CD1/2 Second Track.mp3`
/// - `CD2/1 Third Track.mp3`
/// - `CD2/2 Fourth Track.mp3`
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn transcode_rename_tracks_multi_disc() {
    let snapshot = rename_tracks_helper(AlbumConfig::multi_disc()).await;
    assert_yaml_snapshot!(snapshot);
}

/// Test `rename_tracks` with double-digit track numbers.
///
/// Expected output filenames: `01 Track One.mp3` through `10 Track Ten.mp3`
/// (two-digit padding since max track is 10)
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn transcode_rename_tracks_double_digit_padding() {
    let snapshot = rename_tracks_helper(AlbumConfig::double_digit_tracks()).await;
    assert_yaml_snapshot!(snapshot);
}

/// Test `rename_tracks` with vinyl-style track numbers (A1, A2, B1, B2).
///
/// The `fix_track_numbering` logic converts vinyl notation to proper track/disc numbers
/// in the cached ID3 tags, so renamed files use numeric tracks with CD subfolders.
#[tokio::test]
#[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
async fn transcode_rename_tracks_vinyl_numbering() {
    let snapshot = rename_tracks_helper(AlbumConfig::vinyl_tracks()).await;
    assert_yaml_snapshot!(snapshot);
}

async fn rename_tracks_helper(config: AlbumConfig) -> Vec<FileSnapshot> {
    let _ = init_logger();

    // Generate sample files in cached location
    AlbumGenerator::generate(&config)
        .await
        .expect("should generate album");

    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(config)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            allow_existing: false,
            target: vec![_320, V0],
            sox_random_dither: false,
        })
        .with_options(FileOptions {
            rename_tracks: true,
            no_image_compression: true,
            no_png_to_jpg: true,
            max_file_size: 750_000,
            max_pixel_size: 1280,
            jpg_quality: 80,
        })
        .expect_build();

    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    let status = transcoder.execute(&source).await;

    assert!(status.success);
    DirectorySnapshot::new()
        .with_directory(test_dir.output())
        .without_extensions(&["torrent"])
        .create()
        .expect("should read output directory")
}
