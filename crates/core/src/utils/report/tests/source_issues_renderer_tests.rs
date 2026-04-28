use crate::testing_prelude::*;

#[test]
fn source_issues_renderer_render() {
    // Arrange
    let source_dir = PathBuf::from("/content/Artist - Album [2024]");
    let issues = SourceIssuesRenderer::mock(&source_dir);

    // Act
    let output = SourceIssuesRenderer::render(&issues, &source_dir);

    // Assert
    assert_snapshot!(output);
}
