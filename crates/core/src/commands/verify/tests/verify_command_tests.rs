use crate::testing_prelude::*;

#[tokio::test]
async fn verify_command_mocked() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();

    // Act
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    let result = verifier.execute(&source).await.expect("should not fail");

    // Assert
    if !result.verified() {
        for issue in &result.issues {
            eprintln!("Issue: {issue}");
        }
    }
    assert!(result.verified());
    Ok(())
}

#[tokio::test]
async fn verify_command_execute_no_reports_set() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let test_dir = TestDirectory::new();
    let reports_dir = test_dir.reports();
    let content_dir = test_dir.join("tagless_source");
    create_dir(&content_dir)?;
    FlacGenerator::new()
        .with_filename("01 - track.flac")
        .omit_vorbis_comments()
        .generate(&content_dir)
        .await
        .expect("generate tagless flac");
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let mut builder = HostBuilder::new();
    let _ = builder
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(VerifyOptions {
            no_hash_check: true,
            exclude_tags: None,
            no_decode_check: false,
        })
        .with_options(ReportOptions {
            reports_dir: reports_dir.clone(),
            no_reports: true,
        });
    let host = builder.expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let verifier = host.services.get_required::<VerifyCommand>();
    let mut source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    source.directory = content_dir;

    // Act
    let _ = verifier
        .execute(&source)
        .await
        .expect("verify should succeed");

    // Assert
    assert!(!reports_dir.exists(), "expected no reports directory");
    Ok(())
}
