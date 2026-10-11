//! Windows tests for source and output paths over `MAX_PATH`.

use crate::testing_prelude::*;

/// Folder name long enough to push any path containing it over `MAX_PATH`.
const LONG_DIR_NAME_LENGTH: usize = 200;

/// `sox_ng` writes PNGs to an output path over `MAX_PATH`.
#[tokio::test]
async fn spectrogram_command_long_output() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let output = test_dir.output().join("o".repeat(LONG_DIR_NAME_LENGTH));
    let track_count = album.tracks.len();
    let host = HostBuilder::mock()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: output.clone(),
            ..SharedOptions::mock()
        })
        .expect_build();

    // Act
    let result = execute(&host).await;

    // Assert
    assert!(result.is_ok(), "spectrogram should succeed: {result:?}");
    assert_eq!(count_pngs(&output), track_count * 2);
}

/// `sox_ng` reads source FLACs from a path over `MAX_PATH`.
#[tokio::test]
async fn spectrogram_command_long_source() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let content = TempDirectory::create("spectrogram_command_long_source");
    let content_dir = content.join("s".repeat(LONG_DIR_NAME_LENGTH));
    create_dir_all(&content_dir).expect("should create long directory");
    copy_dir(
        &SAMPLE_SOURCES_DIR.join(album.dir_name()),
        &content_dir.join(album.dir_name()),
        false,
    )
    .await
    .expect("should copy source to long directory");
    let track_count = album.tracks.len();
    let host = HostBuilder::mock()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content_dir],
            output: test_dir.output(),
            ..SharedOptions::mock()
        })
        .expect_build();

    // Act
    let result = execute(&host).await;

    // Assert
    assert!(result.is_ok(), "spectrogram should succeed: {result:?}");
    assert_eq!(count_pngs(&test_dir.output()), track_count * 2);
}

async fn execute(host: &Host) -> Result<(), Failure<SpectrogramAction>> {
    let provider = host.services.get_required::<SourceProvider>();
    let generator = host.services.get_required::<SpectrogramCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    generator.execute(&source).await.map(|_| ())
}

fn count_pngs(dir: &Path) -> usize {
    DirectoryReader::new()
        .with_extension("png")
        .read(dir)
        .expect("should read output directory")
        .len()
}
