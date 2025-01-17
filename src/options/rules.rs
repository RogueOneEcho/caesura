use colored::Colorize;
use log::error;
use std::fmt::{Display, Formatter};

use crate::options::*;
#[derive(Debug)]
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
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
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
        };
        output.fmt(formatter)
    }
}

impl OptionRule {
    pub fn show(errors: &Vec<OptionRule>) {
        if !errors.is_empty() {
            error!("{} configuration", "Invalid".bold());
            for error in errors {
                error!("{}", error);
            }
        }
    }
}
