use crate::options::*;
use colored::Colorize;
use log::error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum OptionRule {
    NotSet(String),
    IsEmpty(String),
    UrlNotHttp(String, String),
    UrlInvalidSuffix(String, String),
    DoesNotExist(String, String),
}

impl Display for OptionRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            NotSet(name) => format!("{name} is not set"),
            IsEmpty(name) => format!("{name} must have at least one value"),
            UrlNotHttp(name, value) => {
                format!("{name} must start with https:// or http://: {value}")
            }
            UrlInvalidSuffix(name, value) => {
                format!("{name} must not end with /: {value}")
            }
            DoesNotExist(name, value) => format!("{name} does not exist: {value}"),
        };
        output.fmt(formatter)
    }
}

impl OptionRule {
    pub fn show(errors: &Vec<OptionRule>) {
        if !errors.is_empty() {
            error!("{} configuration", "Invalid".bold().red());
            for error in errors {
                error!("{}", error);
            }
        }
    }
}
