use std::future::Future;
use std::process::Output;

use tokio::process::Command;

use super::{ProcessError, ProcessOutput};

/// Extension trait for running processes and requiring success.
pub trait ProcessExt {
    /// Run the process and require successful exit.
    ///
    /// - Returns [`Output`] on success
    /// - Returns [`ProcessError::Spawn`] if the process fails to start
    /// - Returns [`ProcessError::Failed`] if the process exits with non-zero status
    fn run(&mut self) -> impl Future<Output = Result<Output, ProcessError>> + Send;
}

impl ProcessExt for Command {
    async fn run(&mut self) -> Result<Output, ProcessError> {
        let output = self.output().await.map_err(ProcessError::Spawn)?;
        require_success(output)
    }
}

/// Check that a process output indicates success.
///
/// Use this for processes that use `.spawn()` + `.wait_with_output()` patterns
/// where you can't use [`ProcessExt::run()`].
pub fn require_success(output: Output) -> Result<Output, ProcessError> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(ProcessError::Failed(ProcessOutput::from(output)))
    }
}
