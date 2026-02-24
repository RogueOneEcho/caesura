use colored::Colorize;
use log::{error, warn};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

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
}

impl Display for OptionRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = match self {
            Self::Changed(name, value, details) => {
                format!("{name} use has changed: {value}\n{details}")
            }
            Self::Dependent(this, that) => format!("{this} requires {that} to be set"),
            Self::NotSet(name) => format!("{name} is not set"),
            Self::IsEmpty(name) => format!("{name} must have at least one value"),
            Self::UrlNotHttp(name, value) => {
                format!("{name} must start with https:// or http://: {value}")
            }
            Self::UrlInvalidSuffix(name, value) => {
                format!("{name} must not end with /: {value}")
            }
            Self::DoesNotExist(name, value) => format!("{name} does not exist: {value}"),
            Self::DurationInvalid(name, value) => format!("{name} could not be parsed: {value}"),
            Self::HashInvalid(name, value) => {
                format!("{name} could not be parsed as a hash: {value}")
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
