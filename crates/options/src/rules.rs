use colored::Colorize;
use log::{error, warn};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Validation rule violation for a configuration option.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum OptionRule {
    /// Option usage has changed and the previous value is no longer accepted.
    ///
    /// Fields: option name, rejected value, explanation of the change.
    Changed(String, String, String),
    /// Required option is not set.
    ///
    /// Field: option name.
    NotSet(String),
    /// At least one of a set of options must be set.
    ///
    /// Field: option names.
    AtLeastOne(Vec<String>),
    /// Option requires another option to be set.
    ///
    /// Fields: dependent option name, required option name.
    Dependent(String, String),
    /// Option must have at least one value.
    ///
    /// Field: option name.
    IsEmpty(String),
    /// URL option does not start with `http://` or `https://`.
    ///
    /// Fields: option name, offending URL.
    UrlNotHttp(String, String),
    /// URL option has an invalid trailing `/`.
    ///
    /// Fields: option name, offending URL.
    UrlInvalidSuffix(String, String),
    /// File or directory path does not exist.
    ///
    /// Fields: option name, offending path.
    DoesNotExist(String, String),
    /// Duration string could not be parsed.
    ///
    /// Fields: option name, offending duration string.
    DurationInvalid(String, String),
    /// Hash string could not be parsed.
    ///
    /// Fields: option name, offending hash string.
    HashInvalid(String, String),
    /// Config file could not be deserialized.
    ///
    /// Field: deserialization error message.
    ConfigDeserialize(String),
    /// Option value could not be extracted from CLI arguments.
    ///
    /// Fields: option name, extraction error message.
    CliExtract(String, String),
}

impl Display for OptionRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = match self {
            Self::Changed(name, value, details) => {
                format!("{name} use has changed: {value}\n{details}")
            }
            Self::Dependent(this, that) => format!("{this} requires {that} to be set"),
            Self::NotSet(name) => format!("{name} is not set"),
            Self::AtLeastOne(names) => {
                format!("at least one of {} must be set", names.join(", "))
            }
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
            Self::ConfigDeserialize(details) => {
                format!("config file could not be deserialized: {details}")
            }
            Self::CliExtract(name, details) => {
                format!("{name} could not be extracted from CLI arguments: {details}")
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
