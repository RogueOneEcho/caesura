use crate::testing_prelude::*;

#[tokio::test]
async fn content_verifier_execute_mismatch() -> Result<(), TestError> {
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
    let verifier = host.services.get_required::<ContentVerifier>();
    let mut source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    let empty_dir = test_dir.join("empty");
    create_dir(&empty_dir)?;
    source.directory = empty_dir;

    // Act
    let result = verifier.execute(&source).await;

    // Assert
    let issue = result
        .expect("hash check should not return infrastructure failure")
        .expect("hash check should return an issue");
    assert!(
        matches!(issue, SourceIssue::MissingFile { .. }),
        "expected MissingFile, got: {issue}"
    );
    Ok(())
}
