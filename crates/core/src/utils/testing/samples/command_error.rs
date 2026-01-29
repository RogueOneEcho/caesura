use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::future::Future;
use std::io::Error as IoError;
use std::process::Output;

use tokio::process::Command;

/// Captured output from a failed command.
#[derive(Debug)]
pub struct CommandOutput {
    /// Standard error output, if any.
    pub stderr: Option<String>,
    /// Exit code, if the process exited normally.
    pub code: Option<i32>,
}

impl From<Output> for CommandOutput {
    fn from(output: Output) -> Self {
        Self {
            stderr: to_option_string(&output.stderr),
            code: output.status.code(),
        }
    }
}

impl Display for CommandOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(e) = &self.stderr {
            write!(f, "{e}")?;
        }
        if let Some(code) = self.code {
            write!(f, "Exit code: {code}")?;
        }
        if self.stderr.is_none() && self.code.is_none() {
            write!(f, "An unknown error occurred")?;
        }
        Ok(())
    }
}

fn to_option_string(buffer: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buffer).trim().to_owned();
    if s.is_empty() { None } else { Some(s) }
}

/// Error from running an external command.
#[derive(Debug)]
pub enum CommandError {
    /// Command failed to spawn (e.g., not found, permission denied).
    Spawn(IoError),
    /// Command ran but exited with non-zero status.
    Failed(CommandOutput),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Spawn(e) => write!(f, "{e}"),
            Self::Failed(output) => write!(f, "{output}"),
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(e) => Some(e),
            Self::Failed(_) => None,
        }
    }
}

/// Extension trait for running commands and requiring success.
pub trait CommandExt {
    /// Run the command and require successful exit.
    ///
    /// - Returns [`Output`] on success
    /// - Returns [`CommandError::Spawn`] if the command fails to start
    /// - Returns [`CommandError::Failed`] if the command exits with non-zero status
    fn run(&mut self) -> impl Future<Output = Result<Output, CommandError>> + Send;
}

impl CommandExt for Command {
    async fn run(&mut self) -> Result<Output, CommandError> {
        let output = self.output().await.map_err(CommandError::Spawn)?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(CommandError::Failed(CommandOutput::from(output)))
        }
    }
}
