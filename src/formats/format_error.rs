use std::fmt::{Display, Formatter};

use crate::formats::FormatError::*;

#[derive(Debug)]
pub enum FormatError {
    UnknownFormat(String),
    UnknownEncoding(String),
}

impl Display for FormatError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            UnknownFormat(input) => format!("Unknown format: {input}"),
            UnknownEncoding(input) => format!("Unknown encoding: {input}"),
        };
        message.fmt(formatter)
    }
}
