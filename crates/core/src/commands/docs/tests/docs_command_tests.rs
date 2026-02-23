use crate::testing_prelude::*;
use insta::assert_snapshot;

/// Test that `DocsCommand::render` produces valid markdown documentation.
#[test]
fn docs_command_renders_markdown() {
    let output = DocsCommand::render();

    assert_snapshot!(output);
}

/// Test that OPTIONS.md matches the output of `DocsCommand::render`.
#[test]
fn options_md_matches_docs_command() {
    let expected = DocsCommand::render();
    let actual = include_str!("../../../../../../docs/OPTIONS.md");

    assert_eq!(
        actual, expected,
        "OPTIONS.md is out of date. Run `caesura docs > OPTIONS.md` to update it."
    );
}
