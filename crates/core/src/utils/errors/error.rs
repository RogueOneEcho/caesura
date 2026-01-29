use rogue_logging::Error;
use tokio::task::JoinError;

use crate::utils::*;

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

#[allow(clippy::wildcard_enum_match_arm)]
#[allow(clippy::absolute_paths)]
pub fn command_error(error: std::io::Error, action: &str, program: &str) -> Error {
    match error.kind() {
        std::io::ErrorKind::NotFound => Error {
            action: action.to_owned(),
            message: format!("Could not find dependency: {program}"),
            ..Error::default()
        },
        _ => io_error(error, action),
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

pub fn output_error(error: CommandError, action: &str, domain: &str) -> Error {
    Error {
        action: action.to_owned(),
        message: error.to_string(),
        domain: Some(domain.to_owned()),
        ..Error::default()
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
