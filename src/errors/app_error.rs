use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error as IOError;

use colored::Colorize;
use log::{error, trace};
use reqwest::StatusCode;
use tokio::task::JoinError;

use crate::errors::app_error::Reason::{Explained, External, Unexpected};
use crate::errors::CommandError;

pub struct AppError {
    action: String,
    reason: Reason,
    pub backtrace: Backtrace,
}

enum Reason {
    Explained(String),
    External(String, Box<dyn Error + Send + Sync>),
    Unexpected(String, String, String),
}

impl AppError {
    #[must_use]
    pub fn else_explained(action: &str, explanation: String) -> AppError {
        Self {
            action: action.to_owned(),
            reason: Explained(explanation),
            backtrace: Backtrace::force_capture(),
        }
    }

    pub fn explained<T>(action: &str, explanation: String) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            reason: Explained(explanation),
            backtrace: Backtrace::force_capture(),
        })
    }

    fn external<T>(
        action: &str,
        domain: &str,
        error: Box<dyn Error + Send + Sync>,
    ) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            reason: External(domain.to_owned(), error),
            backtrace: Backtrace::force_capture(),
        })
    }

    pub fn unexpected<T>(
        action: &str,
        explanation: &str,
        expected: String,
        actual: String,
    ) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            reason: Unexpected(explanation.to_owned(), expected, actual),
            backtrace: Backtrace::force_capture(),
        })
    }

    pub fn claxon<T>(error: claxon::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "FLAC", Box::new(error))
    }

    pub fn command<T>(error: CommandError, action: &str, domain: &str) -> Result<T, AppError> {
        Self::external(action, domain, Box::new(error))
    }

    pub fn io<T>(error: IOError, action: &str) -> Result<T, AppError> {
        Self::external(action, "file system", Box::new(error))
    }

    pub fn request<T>(error: reqwest::Error, action: &str) -> Result<T, AppError> {
        let domain = if let Some(code) = error.status() {
            code.canonical_reason().unwrap_or("API")
        } else {
            "API"
        };
        Self::external(action, domain, Box::new(error))
    }

    pub fn response<T>(status_code: StatusCode, action: &str) -> Result<T, AppError> {
        let status = status_code.canonical_reason().unwrap_or("unknown");
        Self::explained(action, format!("Received a {status} response"))
    }

    pub fn tag<T>(error: audiotags::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "audio tag", Box::new(error))
    }

    pub fn task<T>(error: JoinError, action: &str) -> Result<T, AppError> {
        Self::external(action, "task", Box::new(error))
    }

    pub fn deserialization<T>(error: serde_json::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "deserialization", Box::new(error))
    }

    pub fn lines(&self) -> Vec<String> {
        match &self.reason {
            Explained(explanation) => vec![
                format!("{} to {}", "Failed".bold().red(), self.action),
                format!("{explanation}"),
            ],
            External(domain, error) => vec![
                format!("{} to {}", "Failed".bold().red(), self.action),
                format!("A {domain} error occured"),
                format!("{error}"),
            ],
            Unexpected(explanation, expected, actual) => vec![
                format!("{} to {}", "Failed".bold().red(), self.action),
                format!("{explanation}"),
                format!("Expected: {expected}"),
                format!("Actual: {actual}"),
            ],
        }
    }

    pub fn log(&self) {
        for line in self.lines() {
            error!("{line}");
        }
        if matches!(self.backtrace.status(), BacktraceStatus::Captured) {
            trace!("Backtrace:\n{}", self.backtrace);
        }
    }
}

impl Debug for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.lines().join("\n"))
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.lines().join("\n"))
    }
}

impl Error for AppError {}
