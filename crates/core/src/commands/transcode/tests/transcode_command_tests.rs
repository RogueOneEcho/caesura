use crate::testing_prelude::*;
use std::fs::OpenOptions;

#[tokio::test]
async fn transcode_command_flac16_441() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC16_441).await;
    let snapshot = normalize_snapshots!(snapshot);
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
async fn transcode_command_flac16_48() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC16_48).await;
    let snapshot = normalize_snapshots!(snapshot);
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
async fn transcode_command_flac24_441() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_441).await;
    let snapshot = normalize_snapshots!(snapshot);
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
async fn transcode_command_flac24_48() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_48).await;
    let snapshot = normalize_snapshots!(snapshot);
    assert_yaml_snapshot!(snapshot);
}

#[tokio::test]
async fn transcode_command_flac24_96() {
    let snapshot = transcode_command_helper(SampleFormat::FLAC24_96).await;
    let snapshot = normalize_snapshots!(snapshot);
    assert_yaml_snapshot!(snapshot);
}

/// A non-zero `flac` decode exit must fail the transcode rather than produce a partial output.
///
/// Truncating a source FLAC drops trailing frames so `flac` exits with `LOST_SYNC`, yet `lame`
/// still encodes the partial stream and exits zero. The command must surface the decode failure
/// so the release is not marked transcoded.
#[tokio::test]
async fn transcode_command_truncated_source() {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let content = TempDirectory::create("transcode_command_truncated_source");
    let source_dir = content.join(album.dir_name());
    copy_dir(
        &SAMPLE_SOURCES_DIR.join(album.dir_name()),
        &source_dir,
        false,
    )
    .await
    .expect("should copy source to isolated directory");
    let track = album.tracks.first().expect("album should have a track");
    truncate_to_half(&source_dir.join(album.track_filename(track)));
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(SharedOptions {
            content: vec![content.to_path_buf()],
            output: test_dir.output(),
            ..SharedOptions::mock()
        })
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");

    // Act
    let result = transcoder.execute(&source).await;

    // Assert
    let error = result
        .err()
        .expect("truncated decode should fail the transcode");
    assert!(
        chain_contains_decode(&error),
        "decode failure should appear in the error chain, got: {error}"
    );
}

/// Whether a [`TranscodeAction::Decode`] failure appears in the error source chain.
///
/// The command wraps the job's decode failure as [`TranscodeAction::ExecuteRunner`], so the
/// original action is nested rather than top-level.
fn chain_contains_decode(error: &Failure<TranscodeAction>) -> bool {
    let mut source: Option<&(dyn Error + 'static)> = Some(error);
    while let Some(current) = source {
        if current
            .downcast_ref::<Failure<TranscodeAction>>()
            .is_some_and(|failure| failure.action() == &TranscodeAction::Decode)
        {
            return true;
        }
        source = current.source();
    }
    false
}

/// Truncate a file to half its length, dropping trailing bytes.
#[expect(clippy::integer_division, reason = "halve to truncate")]
fn truncate_to_half(path: &Path) {
    let length = metadata(path).expect("should read metadata").len();
    OpenOptions::new()
        .write(true)
        .open(path)
        .expect("should open file")
        .set_len(length / 2)
        .expect("should truncate file");
}

async fn transcode_command_helper(format: SampleFormat) -> Vec<FileSnapshot> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(format).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let transcoder = host.services.get_required::<TranscodeCommand>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");

    // Act
    let result = transcoder.execute(&source).await;

    // Assert
    assert!(result.is_ok(), "transcode should succeed");
    DirectorySnapshot::new()
        .with_directory(test_dir.output())
        .without_extensions(&["torrent"])
        .create()
        .expect("should read output directory")
}
