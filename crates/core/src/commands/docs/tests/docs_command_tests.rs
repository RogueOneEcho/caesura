use insta::assert_snapshot;

use crate::commands::DocsCommand;

/// Test that `DocsCommand::render` produces valid markdown documentation.
#[test]
fn docs_command_renders_markdown() {
    let output = DocsCommand::render();

    assert_snapshot!(output);
}
