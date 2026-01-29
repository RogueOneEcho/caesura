use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[allow(dead_code)]
pub struct CommandError {
    pub stderr: String,
    pub stdout: String,
    pub exit_code: Option<i32>,
    pub exit_signal: Option<i32>,
    pub exit_stopped_signal: Option<i32>,
}

impl Debug for CommandError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.stderr.is_empty() {
            write!(formatter, "{}", self.stderr)
        } else if !self.stdout.is_empty() {
            write!(formatter, "{}", self.stdout)
        } else {
            write!(formatter, "unexplained failure")
        }
    }
}

impl Display for CommandError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.stderr.is_empty() {
            write!(formatter, "{}", self.stderr)
        } else if !self.stdout.is_empty() {
            write!(formatter, "{}", self.stdout)
        } else {
            write!(formatter, "unexplained failure")
        }
    }
}

impl Error for CommandError {}
