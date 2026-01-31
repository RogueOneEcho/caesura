use crate::prelude::*;
use std::io::ErrorKind;
use tokio::task::JoinError;

#[allow(clippy::absolute_paths)]
pub fn error(action: &str, message: String) -> Error {
    Error {
        action: action.to_owned(),
        message,
        ..Error::default()
    }
}

pub fn claxon_error(error: claxon::Error, action: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some("FLAC".to_owned()),
        ..Error::default()
    }
}

#[allow(clippy::absolute_paths)]
pub fn io_error(error: std::io::Error, action: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some("file system".to_owned()),
        ..Error::default()
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
pub fn process_error(error: ProcessError, action: &str, domain: &str) -> Error {
    match &error {
        ProcessError::Spawn(io_err) if io_err.kind() == ErrorKind::NotFound => Error {
            action: action.to_owned(),
            message: format!("Could not find dependency: {domain}"),
            ..Error::default()
        },
        _ => Error {
            action: action.to_owned(),
            message: error.to_string(),
            domain: Some(domain.to_owned()),
            ..Error::default()
        },
    }
}

pub fn task_error(error: JoinError, action: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some("task".to_owned()),
        ..Error::default()
    }
}

pub fn json_error(error: serde_json::Error, action: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some("deserialization".to_owned()),
        ..Error::default()
    }
}

pub fn yaml_error(error: serde_yaml::Error, action: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some("deserialization".to_owned()),
        ..Error::default()
    }
}
