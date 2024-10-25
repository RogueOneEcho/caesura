use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use colored::Colorize;
use log::{error, trace};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;

use crate::errors::CommandError;

#[derive(Default, Deserialize, Serialize)]
pub struct AppError {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip)]
    pub backtrace: Option<Backtrace>,
}

impl AppError {
    #[must_use]
    pub fn else_explained(action: &str, message: String) -> AppError {
        Self {
            action: action.to_owned(),
            message,
            backtrace: get_backtrace(),
            ..Self::default()
        }
    }

    pub fn explained<T>(action: &str, message: String) -> Result<T, AppError> {
        Err(Self::else_explained(action, message))
    }

    pub fn else_external(action: &str, domain: &str, message: String) -> AppError {
        Self {
            action: action.to_owned(),
            domain: Some(domain.to_owned()),
            message,
            backtrace: get_backtrace(),
            ..Self::default()
        }
    }

    pub fn external<T>(action: &str, domain: &str, message: String) -> Result<T, AppError> {
        Err(Self::else_external(action, domain, message))
    }

    pub fn unexpected<T>(
        action: &str,
        message: &str,
        expected: String,
        actual: String,
    ) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            message: message.to_owned(),
            actual: Some(actual),
            expected: Some(expected),
            backtrace: get_backtrace(),
            ..Self::default()
        })
    }

    pub fn claxon<T>(error: claxon::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "FLAC", format!("{error}"))
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[allow(clippy::absolute_paths)]
    pub fn command<T>(error: std::io::Error, action: &str, program: &str) -> Result<T, AppError> {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                Self::explained(action, format!("Could not find dependency: {program}"))
            }
            _ => Self::io(error, action),
        }
    }

    #[allow(clippy::absolute_paths)]
    pub fn io<T>(error: std::io::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "file system", format!("{error}"))
    }

    pub fn output<T>(error: CommandError, action: &str, domain: &str) -> Result<T, AppError> {
        Self::external(action, domain, format!("{error}"))
    }

    pub fn request<T>(error: reqwest::Error, action: &str) -> Result<T, AppError> {
        let domain = if let Some(code) = error.status() {
            code.canonical_reason().unwrap_or("API")
        } else {
            "API"
        };
        Self::external(action, domain, format!("{error}"))
    }

    pub fn response<T>(status_code: StatusCode, action: &str) -> Result<T, AppError> {
        let status = status_code.canonical_reason().unwrap_or("unknown");
        Self::explained(action, format!("Received a {status} response"))
    }

    pub fn tag<T>(error: audiotags::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "audio tag", format!("{error}"))
    }

    pub fn task<T>(error: JoinError, action: &str) -> Result<T, AppError> {
        Self::external(action, "task", format!("{error}"))
    }

    pub fn json<T>(error: serde_json::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "deserialization", format!("{error}"))
    }

    pub fn yaml<T>(error: serde_yaml::Error, action: &str) -> Result<T, AppError> {
        Self::external(action, "deserialization", format!("{error}"))
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("{} to {}", "Failed".bold(), self.action));
        lines.push(self.message.clone());
        if let Some(actual) = &self.actual {
            lines.push(format!("Actual: {actual}"));
        }
        if let Some(expected) = &self.expected {
            lines.push(format!("Expected: {expected}"));
        }
        lines
    }

    pub fn log(&self) {
        for line in self.lines() {
            error!("{line}");
        }
        if let Some(backtrace) = &self.backtrace {
            trace!("Backtrace:\n{backtrace}");
        }
    }
}

impl Debug for AppError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.lines().join("\n"))
    }
}

impl Display for AppError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.lines().join("\n"))
    }
}
impl Error for AppError {}

impl Clone for AppError {
    fn clone(&self) -> Self {
        Self {
            action: self.action.clone(),
            domain: self.domain.clone(),
            message: self.message.clone(),
            actual: self.actual.clone(),
            expected: self.expected.clone(),
            status_code: self.status_code,
            backtrace: None,
        }
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
fn get_backtrace() -> Option<Backtrace> {
    let backtrace = Backtrace::capture();
    match backtrace.status() {
        BacktraceStatus::Captured => Some(backtrace),
        _ => None,
    }
}
