use crate::testing_prelude::*;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

/// Delay between file operations to ensure filesystem modification times differ.
///
/// When verifying that files were not recreated, we compare modification times before and after
/// an operation. This delay ensures that if a file were recreated, it would have a detectably
/// different modification time. The filesystem's timestamp resolution varies by OS and filesystem
/// (e.g., ext4 has ~1ms resolution, NTFS ~100ns, but some systems round to seconds).
const MODIFICATION_TIME_WAIT: Duration = Duration::from_millis(50);

/// Test that transcode only creates the tracker-specific torrent file.
#[tokio::test]
async fn transcode_creates_only_indexed_torrent() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // Act
    let status = transcoder.execute(&source).await;

    // Assert
    assert!(status.success);
    let indexed_torrent = paths.get_torrent_path(&source, TargetFormat::_320);
    assert!(
        indexed_torrent.exists(),
        "Indexed torrent should exist: {}",
        indexed_torrent.display()
    );
    let filename = indexed_torrent
        .file_name()
        .expect("should have filename")
        .to_string_lossy();
    assert!(
        filename.ends_with(".red.torrent"),
        "Torrent should have indexer suffix: {filename}"
    );
}

/// Test that `get_or_duplicate_existing_torrent_path` returns path when it exists.
#[tokio::test]
async fn get_or_duplicate_returns_path_when_exists() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    let status = transcoder.execute(&source).await;
    assert!(status.success);

    // Act
    let result = paths
        .get_or_duplicate_existing_torrent_path(&source, TargetFormat::_320)
        .await?;

    // Assert
    assert!(result.is_some(), "Should find indexed torrent");
    let path = result.expect("checked above");
    let filename = path.file_name().expect("should have name").to_string_lossy();
    assert!(
        filename.ends_with(".red.torrent"),
        "Should return indexed path: {filename}"
    );

    Ok(())
}

/// Test that `get_or_duplicate` creates torrent from another tracker's torrent.
#[tokio::test]
async fn get_or_duplicate_creates_from_other_tracker() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    // First create a transcode with RED indexer
    let host_red = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host_red.services.get_required::<SourceProvider>();
    let transcoder = host_red.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    let status = transcoder.execute(&source).await;
    assert!(status.success);

    // Now create a new host with OPS indexer, using same output directory
    let host_ops = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: test_dir.output(),
            indexer: Some("ops".to_owned()),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let paths_ops = host_ops.services.get_required::<PathManager>();

    // Act - should find .red.torrent and duplicate to .ops.torrent
    let result = paths_ops
        .get_or_duplicate_existing_torrent_path(&source, TargetFormat::_320)
        .await?;

    // Assert
    assert!(result.is_some(), "Should create from RED torrent");
    let path = result.expect("checked above");
    let filename = path.file_name().expect("should have name").to_string_lossy();
    assert!(
        filename.ends_with(".ops.torrent"),
        "Should create OPS torrent: {filename}"
    );
    assert!(path.exists(), "OPS torrent file should exist");

    Ok(())
}

/// Test that `get_or_duplicate` returns None when no torrent exists.
#[tokio::test]
async fn get_or_duplicate_returns_none_when_missing() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // Act - no transcode performed, no torrent files exist
    let result = paths
        .get_or_duplicate_existing_torrent_path(&source, TargetFormat::_320)
        .await?;

    // Assert
    assert!(result.is_none(), "Should return None when no torrent exists");

    Ok(())
}

/// Test that torrent filename includes format and indexer.
#[tokio::test]
async fn torrent_filename_includes_format_and_indexer() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // Act
    let status = transcoder.execute(&source).await;
    assert!(status.success);

    // Assert
    let torrent_path = paths.get_torrent_path(&source, TargetFormat::_320);
    let filename = torrent_path
        .file_name()
        .expect("should have filename")
        .to_string_lossy();
    assert!(
        filename.contains("[WEB 320]"),
        "Filename should contain format: {filename}"
    );
    assert!(
        filename.ends_with(".red.torrent"),
        "Filename should end with indexer: {filename}"
    );
    assert!(
        filename.contains("Test Artist"),
        "Filename should contain artist: {filename}"
    );
}

/// Test that multiple target formats each get their own torrent file.
#[tokio::test]
async fn transcode_creates_torrents_for_each_format() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320, TargetFormat::V0],
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // Act
    let status = transcoder.execute(&source).await;

    // Assert
    assert!(status.success);
    for target in [TargetFormat::_320, TargetFormat::V0] {
        let torrent_path = paths.get_torrent_path(&source, target);
        assert!(
            torrent_path.exists(),
            "{target} torrent should exist: {}",
            torrent_path.display()
        );
    }
}

/// Test that switching indexer finds existing torrent and skips transcoding.
#[tokio::test]
async fn transcode_skips_when_other_tracker_torrent_exists() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    // First create a transcode with RED indexer
    let host_red = HostBuilder::new()
        .with_mock_api(album.clone())
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host_red.services.get_required::<SourceProvider>();
    let transcoder_red = host_red.services.get_required::<TranscodeCommand>();
    let paths_red = host_red.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    let status = transcoder_red.execute(&source).await;
    assert!(status.success);

    let red_torrent = paths_red.get_torrent_path(&source, TargetFormat::_320);
    assert!(red_torrent.exists(), "RED torrent should exist");

    // Record modification time of a transcoded file to verify no re-transcoding
    let transcode_dir = paths_red.get_transcode_target_dir(&source, TargetFormat::_320);
    let mp3_file = fs::read_dir(&transcode_dir)
        .expect("should read transcode dir")
        .filter_map(Result::ok)
        .find(|e| e.path().extension().is_some_and(|ext| ext == "mp3"))
        .expect("should have mp3 file");
    let mp3_modified_before = fs::metadata(mp3_file.path())
        .expect("should get metadata")
        .modified()
        .expect("should get mtime");
    sleep(MODIFICATION_TIME_WAIT).await;

    // Now create a new host with OPS indexer
    let host_ops = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: test_dir.output(),
            indexer: Some("ops".to_owned()),
            ..SharedOptions::mock()
        })
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let transcoder_ops = host_ops.services.get_required::<TranscodeCommand>();
    let paths_ops = host_ops.services.get_required::<PathManager>();

    // Act - transcode with OPS should find RED torrent and skip
    let status = transcoder_ops.execute(&source).await;

    // Assert
    assert!(status.success);
    let ops_torrent = paths_ops.get_torrent_path(&source, TargetFormat::_320);
    assert!(
        ops_torrent.exists(),
        "OPS torrent should be created from RED: {}",
        ops_torrent.display()
    );
    assert!(red_torrent.exists(), "RED torrent should still exist");

    // Verify transcoding was skipped by checking mp3 wasn't modified
    let mp3_modified_after = fs::metadata(mp3_file.path())
        .expect("should get metadata")
        .modified()
        .expect("should get mtime");
    assert_eq!(
        mp3_modified_before, mp3_modified_after,
        "MP3 file should not be recreated - transcoding should be skipped"
    );
}

/// Test that re-running transcode skips when torrent exists.
#[tokio::test]
async fn transcode_skips_when_torrent_exists() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(TargetOptions {
            target: vec![TargetFormat::_320],
            ..TargetOptions::default()
        })
        .build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let paths = host.services.get_required::<PathManager>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("Source provider should not fail");

    // First transcode
    let status1 = transcoder.execute(&source).await;
    assert!(status1.success);

    let torrent_path = paths.get_torrent_path(&source, TargetFormat::_320);
    let modified_before = fs::metadata(&torrent_path)
        .expect("torrent should exist")
        .modified()
        .expect("should get modified time");
    sleep(MODIFICATION_TIME_WAIT).await;

    // Act - second transcode should skip
    let status2 = transcoder.execute(&source).await;

    // Assert
    assert!(status2.success);
    let modified_after = fs::metadata(&torrent_path)
        .expect("torrent should exist")
        .modified()
        .expect("should get modified time");
    assert_eq!(
        modified_before, modified_after,
        "Torrent file should not be recreated"
    );
}
