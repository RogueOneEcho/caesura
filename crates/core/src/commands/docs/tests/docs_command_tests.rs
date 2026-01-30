use crate::testing_prelude::*;
use insta::assert_snapshot;

/// Test that `DocsCommand::render` produces valid markdown documentation.
#[test]
fn docs_command_renders_markdown() {
    let output = DocsCommand::render();

    assert_snapshot!(output);
}
