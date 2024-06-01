use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error as IOError;

use colored::Colorize;
use reqwest::Response;
use tokio::task::JoinError;
use crate::errors::CommandError;

pub struct AppError {
    action: String,
    kind: String,
    expected: Option<String>,
    actual: Option<String>,
    source: Option<Box<dyn Error>>,
    backtrace: Option<Backtrace>,
}

impl AppError {
    pub fn new(action: &str, kind: &str) -> Self {
        Self {
            action: action.to_owned(),
            kind: kind.to_owned(),
            expected: None,
            actual: None,
            source: None,
            backtrace: Some(Backtrace::force_capture()),
        }
    }

    pub fn claxon<T>(error: claxon::Error, action: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: "FLAC".to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn command<T>(error: CommandError, action: &str, kind: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: kind.to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn io<T>(error: IOError, action: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: "file system".to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn request<T>(error: reqwest::Error, action: &str) -> Result<T, AppError> {
        let kind = if let Some(code) = error.status() {
            code.canonical_reason().unwrap_or("API")
        } else {
            "API"
        };
        Err(Self {
            action: action.to_owned(),
            kind: kind.to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn response<T>(response: Response, action: &str) -> Result<T, AppError> {  
        
        let kind = response.status().canonical_reason().unwrap_or("API");
        Err(Self {
            action: action.to_owned(),
            kind: kind.to_owned(),
            expected: None,
            actual: None,
            source: None,
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn tag<T>(error: audiotags::Error, action: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: "audio tag".to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }

    pub fn task<T>(error: JoinError, action: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: "task".to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }
    
    pub fn deserialization<T>(error: serde_json::Error, action: &str) -> Result<T, AppError> {
        Err(Self {
            action: action.to_owned(),
            kind: "deserialization".to_owned(),
            expected: None,
            actual: None,
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::force_capture()),
        })
    }
}

impl Debug for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut message = format!(
            "{} to {}\nA {} error occured",
            "Failed".bold().red(),
            self.action,
            self.kind
        );
        if let Some(source) = &self.source {
            message = format!("{message} due to {source}");
        }
        write!(formatter, "{}", message)
    }
}

impl Error for AppError {}
