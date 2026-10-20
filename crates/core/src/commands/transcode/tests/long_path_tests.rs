//! Windows tests for source and output paths over `MAX_PATH`.

use crate::testing_prelude::*;

/// Folder name long enough to push any path containing it over `MAX_PATH`.
const LONG_DIR_NAME_LENGTH: usize = 200;

/// `sox_ng` resamples and decodes from a long source path, then `sox_ng` and `lame` write to a
/// long output path.
#[tokio::test]
async fn transcode_command_long_paths_flac24() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::FLAC24_441).await;
    let test_dir = TestDirectory::new();
    let content = TempDirectory::create("transcode_command_long_paths_flac24");
    let content_dir = copy_to_long_dir(&album, &content).await;
    let output = test_dir.output().join("o".repeat(LONG_DIR_NAME_LENGTH));
    let track_count = album.tracks.len();
    let host = build_host(album, &test_dir, content_dir, output.clone()).await;

    // Act
    let result = execute(&host).await;

    // Assert
    assert!(result.is_ok(), "transcode should succeed: {result:?}");
    assert_eq!(count_files(&output, "mp3"), track_count * 2);
    assert_eq!(count_files(&output, "flac"), track_count);
}

/// `flac` decodes from a short source path and `lame` writes to a long output path.
#[tokio::test]
async fn transcode_command_long_output_flac16() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::FLAC16_441).await;
    let test_dir = TestDirectory::new();
    let output = test_dir.output().join("o".repeat(LONG_DIR_NAME_LENGTH));
    let track_count = album.tracks.len();
    let host = build_host(album, &test_dir, SAMPLE_SOURCES_DIR.clone(), output.clone()).await;

    // Act
    let result = execute(&host).await;

    // Assert
    assert!(result.is_ok(), "transcode should succeed: {result:?}");
    assert_eq!(count_files(&output, "mp3"), track_count * 2);
}

/// `flac` can't read a long source path, so the transcode fails with a clear error.
#[tokio::test]
async fn transcode_command_long_source_flac16() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::FLAC16_441).await;
    let test_dir = TestDirectory::new();
    let content = TempDirectory::create("transcode_command_long_source_flac16");
    let content_dir = copy_to_long_dir(&album, &content).await;
    let host = build_host(album, &test_dir, content_dir, test_dir.output()).await;

    // Act
    let result = execute(&host).await;

    // Assert
    let error = result.expect_err("long source path should fail the transcode");
    assert!(
        chain_contains(&error, TranscodeAction::SourcePathTooLong),
        "source path too long should appear in the error chain, got: {error}"
    );
}

/// Copy the album source into a long folder inside `content` and return the long folder.
async fn copy_to_long_dir(album: &AlbumConfig, content: &TempDirectory) -> PathBuf {
    let content_dir = content.join("s".repeat(LONG_DIR_NAME_LENGTH));
    create_dir_all(&content_dir).expect("should create long directory");
    copy_dir(
        &SAMPLE_SOURCES_DIR.join(album.dir_name()),
        &content_dir.join(album.dir_name()),
        false,
    )
    .await
    .expect("should copy source to long directory");
    content_dir
}

async fn build_host(
    album: AlbumConfig,
    test_dir: &TestDirectory,
    content_dir: PathBuf,
    output: PathBuf,
) -> Host {
    HostBuilder::mock()
        .with_mock_api(album)
        .with_test_options(test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir],
            output,
            ..SharedOptions::mock()
        })
        .expect_build()
}

async fn execute(host: &Host) -> Result<(), Failure<TranscodeAction>> {
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    transcoder.execute(&source).await.map(|_| ())
}

fn count_files(dir: &Path, extension: &str) -> usize {
    DirectoryReader::new()
        .with_extension(extension)
        .read(dir)
        .expect("should read output directory")
        .len()
}

/// Whether a failure with `action` appears in the error source chain.
fn chain_contains(error: &Failure<TranscodeAction>, action: TranscodeAction) -> bool {
    error
        .chain()
        .filter_map(|source| source.downcast_ref::<Failure<TranscodeAction>>())
        .any(|failure| failure.action() == &action)
}
