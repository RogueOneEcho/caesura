use crate::commands::CommandArguments::{self, *};
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for output names
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct NameOptions {
    /// Override the output name prefix.
    ///
    /// When set, this value replaces the default `Artist - Album (Remaster) [Year]`
    /// prefix. The format suffix (`[CD FLAC]`, etc.) is still appended automatically.
    /// When `--name-template` is set, available as `{{ name }}` instead.
    #[arg(long)]
    pub name: Option<String>,

    /// Minijinja template for full control over output naming.
    ///
    /// Unlike `--name`, the format suffix is **not** appended automatically.
    ///
    /// Refer to [NAME-TEMPLATE.md](NAME-TEMPLATE.md) for details on the template syntax and available variables.
    ///
    /// Requires `--experimental-name-template`.
    #[arg(long)]
    pub name_template: Option<String>,

    /// Enable experimental minijinja template rendering for `--name-template`.
    ///
    /// Required when using `--name-template`. Signals that the template naming
    /// system is still a work in progress and may change.
    #[arg(long)]
    pub experimental_name_template: bool,
}

impl OptionsContract for NameOptions {
    type Partial = NameOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(name) = &self.name {
            let has_template_syntax = name.contains("{{")
                || name.contains("}}")
                || name.contains("{%")
                || name.contains("%}");
            if has_template_syntax {
                errors.push(TemplateSyntaxNotAllowed("name".to_owned()));
            }
            if !Sanitizer::validate(name) {
                errors.push(RestrictedChars("name".to_owned()));
            }
        }
        if let Some(template) = &self.name_template {
            if !Sanitizer::validate(template) {
                errors.push(RestrictedChars("template".to_owned()));
            }
            if !self.experimental_name_template {
                errors.push(Dependent(
                    "name_template".to_owned(),
                    "experimental_name_template".to_owned(),
                ));
            }
            TemplateEngine::validate(template, errors);
        }
    }
}

impl FromArgs for NameOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(
                Batch { name, .. }
                | Spectrogram { name, .. }
                | TestName { name, .. }
                | Transcode { name, .. }
                | Upload { name, .. }
                | Verify { name, .. },
            ) => Some(name.clone()),
            _ => None,
        }
    }
}
