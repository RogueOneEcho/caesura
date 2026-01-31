use std::fmt::{Display, Formatter, Result as FmtResult};
use std::process::Output;

/// Captured output from a failed process.
#[derive(Debug)]
pub struct ProcessOutput {
    /// Standard error output, if any.
    pub stderr: Option<String>,
    /// Exit code, if the process exited normally.
    pub code: Option<i32>,
}

impl From<Output> for ProcessOutput {
    fn from(output: Output) -> Self {
        Self {
            stderr: to_option_string(&output.stderr),
            code: output.status.code(),
        }
    }
}

impl Display for ProcessOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match (&self.stderr, self.code) {
            (Some(e), Some(code)) => write!(f, "{e}\nExit code: {code}"),
            (Some(e), None) => write!(f, "{e}"),
            (None, Some(code)) => write!(f, "Exit code: {code}"),
            (None, None) => write!(f, "An unknown error occurred"),
        }
    }
}

fn to_option_string(buffer: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buffer).trim().to_owned();
    if s.is_empty() { None } else { Some(s) }
}
