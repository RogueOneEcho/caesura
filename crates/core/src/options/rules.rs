use crate::prelude::*;
use serde::Serialize;

/// Validation rule violation for a configuration option.
#[derive(Clone, Debug, Serialize)]
pub enum OptionRule {
    Changed(String, String, String),
    NotSet(String),
    Dependent(String, String),
    IsEmpty(String),
    UrlNotHttp(String, String),
    UrlInvalidSuffix(String, String),
    DoesNotExist(String, String),
    DurationInvalid(String, String),
    HashInvalid(String, String),
    TemplateSyntax(String, String),
    TemplateSyntaxNotAllowed(String),
    RestrictedChars(String),
}

impl Display for OptionRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = match self {
            Changed(name, value, details) => format!("{name} use has changed: {value}\n{details}"),
            Dependent(this, that) => format!("{this} requires {that} to be set"),
            NotSet(name) => format!("{name} is not set"),
            IsEmpty(name) => format!("{name} must have at least one value"),
            UrlNotHttp(name, value) => {
                format!("{name} must start with https:// or http://: {value}")
            }
            UrlInvalidSuffix(name, value) => {
                format!("{name} must not end with /: {value}")
            }
            DoesNotExist(name, value) => format!("{name} does not exist: {value}"),
            DurationInvalid(name, value) => format!("{name} could not be parsed: {value}"),
            HashInvalid(name, value) => format!("{name} could not be parsed as a hash: {value}"),
            TemplateSyntax(name, value) => format!("{name} template is invalid: {value}"),
            TemplateSyntaxNotAllowed(name) => {
                format!("{name} must not contain template syntax; use --name-template instead")
            }
            RestrictedChars(name) => {
                format!(
                    "{name} contains characters that are not allowed. Refer to NAME-TEMPLATE.md for details."
                )
            }
        };
        write!(formatter, "{output}")
    }
}

impl OptionRule {
    /// Log all validation errors to the console.
    pub fn show(errors: &Vec<OptionRule>) {
        if !errors.is_empty() {
            error!("{} configuration", "Invalid".bold());
            for error in errors {
                warn!("{error}");
            }
        }
    }
}
