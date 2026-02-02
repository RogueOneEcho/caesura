//! Error types for host building.

use crate::options::OptionRule;
use colored::Colorize;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Error that occurs when building the application host.
#[derive(Debug)]
pub enum BuildError {
    /// Options validation failed.
    Options(Vec<OptionRule>),
    /// Dependency injection container failed to build.
    Container(di::ValidationError),
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BuildError::Options(errors) => {
                writeln!(f, "Configuration is invalid:")?;
                for error in errors {
                    writeln!(f, "{} {error}", "△".yellow())?;
                }
                Ok(())
            }
            BuildError::Container(error) => {
                write!(f, "Failed to build the application: {error}")
            }
        }
    }
}

impl Error for BuildError {}

impl From<Vec<OptionRule>> for BuildError {
    fn from(errors: Vec<OptionRule>) -> Self {
        BuildError::Options(errors)
    }
}

impl From<di::ValidationError> for BuildError {
    fn from(error: di::ValidationError) -> Self {
        BuildError::Container(error)
    }
}
