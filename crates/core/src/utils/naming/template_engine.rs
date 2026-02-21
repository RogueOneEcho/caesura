//! Minijinja template engine for output naming.

use crate::prelude::*;
use minijinja::{Environment, Value};
use serde::Serialize;
use std::sync::LazyLock;

/// Minijinja template engine with custom filters for output naming.
pub(crate) struct TemplateEngine;

static ENGINE: LazyLock<Environment<'static>> = LazyLock::new(create_engine);

impl TemplateEngine {
    /// Validate a template string.
    ///
    /// - Renders with mock metadata to catch syntax errors early.
    pub(crate) fn validate(template: &str, errors: &mut Vec<OptionRule>) {
        let context = NameContext::mock();
        if let Err(e) = ENGINE.render_str(template, &context) {
            errors.push(TemplateSyntax("--name-template".to_owned(), e.to_string()));
        }
    }

    /// Render a minijinja template with the given context.
    #[must_use]
    pub(crate) fn render(template: &str, context: &impl Serialize) -> String {
        let rendered = ENGINE
            .render_str(template, context)
            .expect("template should render after startup validation");
        Sanitizer::execute(rendered)
    }
}

fn create_engine() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("and_join", and_join_filter);
    env.add_filter("limit", limit_filter);
    env
}

fn and_join_filter(items: Vec<String>) -> String {
    and_join(items)
}

fn limit_filter(items: Vec<Value>, n: usize) -> Vec<Value> {
    items.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Template conditional includes remaster title when present.
    #[test]
    fn render_conditional_with_remaster() {
        // Arrange
        let context = NameContext::mock();
        let template = "{{ artist }}{% if edition_title %} ({{ edition_title }}){% endif %}";

        // Act
        let result = TemplateEngine::render(template, &context);

        // Assert
        let title = context
            .metadata
            .edition_title
            .as_ref()
            .expect("mock has edition_title");
        assert_eq!(result, format!("{} ({})", context.metadata.artist, title));
    }

    /// Template conditional omits remaster title when absent.
    #[test]
    fn render_conditional_without_remaster() {
        // Arrange
        let context = NameContext {
            metadata: Metadata {
                edition_title: None,
                ..Metadata::mock()
            },
            ..NameContext::mock()
        };
        let template = "{{ artist }}{% if edition_title %} ({{ edition_title }}){% endif %}";

        // Act
        let result = TemplateEngine::render(template, &context);

        // Assert
        assert_eq!(result, context.metadata.artist);
    }

    /// Valid template produces no validation errors.
    #[test]
    fn validate_valid_template() {
        // Arrange
        let mut errors = Vec::new();

        // Act
        TemplateEngine::validate("{{ artist }} - {{ album }}", &mut errors);

        // Assert
        assert!(errors.is_empty());
    }

    /// Template using `{{ format }}` passes validation.
    #[test]
    fn validate_template_with_format_variable() {
        // Arrange
        let mut errors = Vec::new();

        // Act
        TemplateEngine::validate("{{ artist }} [{{ media }} {{ format }}]", &mut errors);

        // Assert
        assert!(errors.is_empty());
    }

    /// Invalid template syntax produces a `TemplateSyntax` error.
    #[test]
    fn validate_invalid_template() {
        // Arrange
        let mut errors = Vec::new();

        // Act
        TemplateEngine::validate("{{ artist }{% endif %}", &mut errors);

        // Assert
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors.first(), Some(TemplateSyntax(..))));
    }

    /// Verify `limit` filter truncates a list to N items.
    #[test]
    fn limit_filter_truncates_list() {
        // Arrange
        let template = r#"{{ ["a", "b", "c", "d", "e"]|limit(3)|join(", ") }}"#;

        // Act
        let result = TemplateEngine::render(template, &NameContext::mock());

        // Assert
        assert_eq!(result, "a, b, c");
    }

    /// Verify `limit` combined with `and_join`.
    #[test]
    fn limit_filter_with_and_join() {
        // Arrange
        let template = r#"{{ ["Alice", "Bob", "Carol", "Dave"]|limit(3)|and_join }}"#;

        // Act
        let result = TemplateEngine::render(template, &NameContext::mock());

        // Assert
        assert_eq!(result, "Alice, Bob & Carol");
    }

    /// Credit list variables are accessible in templates.
    #[test]
    fn render_credit_list_variables() {
        // Arrange
        let context = NameContext {
            metadata: Metadata::mock_3_artists_1_dj(),
            ..NameContext::mock()
        };
        let template = "{{ artists|and_join }} (mixed by {{ dj|first }})";

        // Act
        let result = TemplateEngine::render(template, &context);

        // Assert
        assert_eq!(
            result,
            "Artist One, Artist Two & Artist Three (mixed by Mock DJ)"
        );
    }

    /// Verify `limit` with N larger than the list returns all items.
    #[test]
    fn limit_filter_larger_than_list() {
        // Arrange
        let template = r#"{{ ["a", "b"]|limit(5)|join(", ") }}"#;

        // Act
        let result = TemplateEngine::render(template, &NameContext::mock());

        // Assert
        assert_eq!(result, "a, b");
    }

    /// The `get_artist` fallback logic from [`Metadata`] can be replicated as a template.
    #[test]
    fn render_get_artist_logic_as_template() {
        // Arrange
        let template = concat!(
            "{% if artists|length > 0 and artists|length <= 2 %}",
            "{{ artists|and_join }}",
            "{% elif dj|length == 1 %}",
            "{{ dj|first }}",
            "{% elif composers|length == 1 %}",
            "{{ composers|first }}",
            "{% elif artists|length == 0 and with|length > 0 and with|length <= 2 %}",
            "{{ with|and_join }}",
            "{% elif artists|length == 0 and with|length == 0 %}",
            "Unknown Artist",
            "{% else %}",
            "Various Artists",
            "{% endif %}",
        );
        let mocks: Vec<Metadata> = vec![
            Metadata::mock(),
            Metadata::mock_2_artists(),
            Metadata::mock_3_artists(),
            Metadata::mock_3_artists_1_dj(),
            Metadata::mock_3_artists_1_composer(),
            Metadata::mock_1_artist_1_composer(),
            Metadata::mock_0_artists_2_guests(),
            Metadata::mock_0_artists(),
        ];
        for metadata in mocks {
            let expected = metadata.artist.clone();
            let context = NameContext {
                metadata,
                ..NameContext::mock()
            };

            // Act
            let result = TemplateEngine::render(template, &context);

            // Assert
            assert_eq!(result, expected);
        }
    }
}
