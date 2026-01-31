use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;

use super::ProcessOutput;

/// Error from running an external process.
#[derive(Debug)]
pub enum ProcessError {
    /// Process failed to spawn (e.g., not found, permission denied).
    Spawn(IoError),
    /// Wait for process completion failed.
    Wait(IoError),
    /// Process ran but exited with non-zero status.
    Failed(ProcessOutput),
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Spawn(e) => write!(f, "Could not start process\n{e}"),
            Self::Wait(e) => write!(f, "Process interrupted\n{e}"),
            Self::Failed(output) => write!(f, "{output}"),
        }
    }
}

impl Error for ProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(e) | Self::Wait(e) => Some(e),
            Self::Failed(_) => None,
        }
    }
}
