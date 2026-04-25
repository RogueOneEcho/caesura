use crate::testing_prelude::*;

#[tokio::test]
async fn report_renderer_render_single_no_tags_issue() {
    // Arrange
    let mut builder = HostBuilder::new();
    let _ = builder.with_options(SharedOptions {
        indexer_url: "https://example.com".to_owned(),
        ..SharedOptions::mock()
    });
    let host = builder.expect_build();
    let renderer = host.services.get_required::<ReportRenderer>();
    let source = Source::mock();
    let issues = vec![SourceIssue::NoTags {
        path: PathBuf::from("/content/Artist - Album [2024]/02 - Track.flac"),
    }];

    // Act
    let output = renderer
        .render(&source, &issues)
        .expect("render should succeed");

    // Assert
    assert_snapshot!(output);
}

#[tokio::test]
async fn report_renderer_render_multiple_issue_types() {
    // Arrange
    let mut builder = HostBuilder::new();
    let _ = builder.with_options(SharedOptions {
        indexer_url: "https://example.com".to_owned(),
        ..SharedOptions::mock()
    });
    let host = builder.expect_build();
    let renderer = host.services.get_required::<ReportRenderer>();
    let source = Source::mock();
    let issues = vec![
        SourceIssue::NoTags {
            path: PathBuf::from("/content/Artist - Album [2024]/02 - Track.flac"),
        },
        SourceIssue::MissingTags {
            path: PathBuf::from("/content/Artist - Album [2024]/10 - Another.flac"),
            tags: vec!["composer".to_owned()],
        },
        SourceIssue::UnnecessaryDirectory {
            prefix: PathBuf::from("CD1"),
        },
    ];

    // Act
    let output = renderer
        .render(&source, &issues)
        .expect("render should succeed");

    // Assert
    assert_snapshot!(output);
}
