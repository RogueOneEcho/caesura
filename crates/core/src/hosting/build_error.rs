//! Error types for host building.

use crate::prelude::*;
use di::ValidationError;

/// Error that occurs when building the application host.
#[derive(Debug)]
pub enum BuildError {
    /// Options validation failed.
    Options(Vec<OptionIssue>),
    /// Dependency injection container failed to build.
    Container(ValidationError),
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

impl From<Vec<OptionIssue>> for BuildError {
    fn from(errors: Vec<OptionIssue>) -> Self {
        BuildError::Options(errors)
    }
}

impl From<ValidationError> for BuildError {
    fn from(error: ValidationError) -> Self {
        BuildError::Container(error)
    }
}
