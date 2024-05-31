use crate::imdl::ImdlError::*;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ImdlError {
    IOFailure(std::io::Error),
    DeserializationFailure(serde_json::Error),
}

impl Display for ImdlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            IOFailure(error) => format!("IO failed: {error}"),
            DeserializationFailure(error) => format!("Deserialization failed: {error}"),
        };
        message.fmt(formatter)
    }
}
