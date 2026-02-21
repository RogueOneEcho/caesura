//! Output name resolution from metadata.

use crate::prelude::*;

pub(crate) const DEFAULT_TEMPLATE: &str = concat!(
    "{% if name %}{{name}}{% else %}",
    "{{ artist }} - {{ album }}",
    "{% if edition_title %} ({{ edition_title }}){% endif %}",
    " [{{ year }}]",
    "{% endif %}",
    " [{{ media }}",
    "{% if spectrogram %} SPECTROGRAMS",
    "{% elif format %} {{ format }}",
    "{% endif %}]",
);

/// Resolve complete output names from options and metadata.
#[injectable]
pub(crate) struct NameResolver {
    options: Ref<NameOptions>,
}

impl NameResolver {
    /// Resolve the output name for a transcode.
    ///
    /// Uses `--name-template` if set, otherwise falls back to [`DEFAULT_TEMPLATE`].
    #[must_use]
    pub(crate) fn transcode(&self, metadata: &Metadata, target: TargetFormat) -> String {
        let context = NameContext {
            metadata: metadata.clone(),
            format: Some(target.get_name().to_owned()),
            name: self.options.name.clone(),
            spectrogram: false,
        };
        self.render(&context)
    }

    /// Resolve the output name for spectrograms.
    #[must_use]
    pub(crate) fn spectrogram(&self, metadata: &Metadata) -> String {
        let context = NameContext {
            metadata: metadata.clone(),
            format: None,
            name: self.options.name.clone(),
            spectrogram: true,
        };
        self.render(&context)
    }

    fn render(&self, context: &NameContext) -> String {
        let template = self
            .options
            .name_template
            .as_deref()
            .unwrap_or(DEFAULT_TEMPLATE);
        TemplateEngine::render(template, context)
    }

    #[cfg(test)]
    fn with_options(options: NameOptions) -> Self {
        Self {
            options: Ref::new(options),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// When no name options are set, resolve uses default template.
    #[test]
    fn resolve_default() {
        // Arrange
        let resolver = NameResolver::with_options(NameOptions::default());
        let metadata = Metadata::mock();

        // Act
        let result = resolver.transcode(&metadata, TargetFormat::Flac);

        // Assert
        let expected = format!("{} [CD FLAC]", SourceName::get(&metadata));
        assert_eq!(result, expected);
    }

    /// Static `--name` is used as prefix with format suffix appended.
    #[test]
    fn resolve_static_override() {
        // Arrange
        let resolver = NameResolver::with_options(NameOptions {
            name: Some("Custom Name".to_owned()),
            ..NameOptions::default()
        });

        // Act
        let result = resolver.transcode(&Metadata::mock(), TargetFormat::_320);

        // Assert
        assert_eq!(result, "Custom Name [CD 320]");
    }

    /// `--name-template` takes precedence over `--name`.
    #[test]
    fn resolve_template_takes_precedence_over_name() {
        // Arrange
        let resolver = NameResolver::with_options(NameOptions {
            name: Some("Static Name".to_owned()),
            name_template: Some("{{ artist }}".to_owned()),
            experimental_name_template: true,
        });

        // Act
        let result = resolver.transcode(&Metadata::mock(), TargetFormat::Flac);

        // Assert
        assert_eq!(result, "Mock Artist");
    }

    /// Template `--name-template` renders metadata variables (no auto suffix).
    #[test]
    fn resolve_template() {
        // Arrange
        let metadata = Metadata::mock();
        let resolver = NameResolver::with_options(NameOptions {
            name_template: Some("{{ artist }} - {{ album }} [{{ year }}]".to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        });

        // Act
        let result = resolver.transcode(&metadata, TargetFormat::Flac);

        // Assert
        assert_eq!(
            result,
            format!(
                "{} - {} [{}]",
                metadata.artist, metadata.album, metadata.year
            )
        );
    }

    /// Template can use the `format` variable.
    #[test]
    fn resolve_template_with_format_variable() {
        // Arrange
        let metadata = Metadata::mock();
        let resolver = NameResolver::with_options(NameOptions {
            name_template: Some("{{ artist }} - {{ album }} [{{ media }} {{ format }}]".to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        });

        // Act
        let result = resolver.transcode(&metadata, TargetFormat::V0);

        // Assert
        assert_eq!(result, "Mock Artist - Test Album [CD V0]");
    }

    /// Spectrogram resolution uses "SPECTROGRAMS" suffix instead of format.
    #[test]
    fn resolve_spectrogram_default() {
        // Arrange
        let resolver = NameResolver::with_options(NameOptions::default());
        let metadata = Metadata::mock();

        // Act
        let result = resolver.spectrogram(&metadata);

        // Assert
        let expected = format!("{} [CD SPECTROGRAMS]", SourceName::get(&metadata));
        assert_eq!(result, expected);
    }

    /// Spectrogram resolution with `--name` override.
    #[test]
    fn resolve_spectrogram_with_name() {
        // Arrange
        let resolver = NameResolver::with_options(NameOptions {
            name: Some("Custom Name".to_owned()),
            ..NameOptions::default()
        });

        // Act
        let result = resolver.spectrogram(&Metadata::mock());

        // Assert
        assert_eq!(result, "Custom Name [CD SPECTROGRAMS]");
    }
}
