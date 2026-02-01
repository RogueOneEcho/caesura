use crate::testing_prelude::*;
use insta::assert_snapshot;

/// Test that `DocsCommand::render` produces valid markdown documentation.
#[test]
fn docs_command_renders_markdown() {
    let output = DocsCommand::render();

    assert_snapshot!(output);
}

/// Test that CONFIG.md matches the output of `DocsCommand::render`.
#[test]
fn config_md_matches_docs_command() {
    let expected = DocsCommand::render();
    let actual = include_str!("../../../../../../CONFIG.md");

    assert_eq!(
        actual, expected,
        "CONFIG.md is out of date. Run `caesura docs > CONFIG.md` to update it."
    );
}
