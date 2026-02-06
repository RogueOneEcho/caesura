use std::future::Future;
use std::process::Output;

use rogue_logging::Failure;
use tokio::process::Command;

use super::{ProcessAction, ProcessOutput};

/// Extension trait for running processes and requiring success.
pub trait ProcessExt {
    /// Run the process and require successful exit.
    ///
    /// - Returns [`Output`] on success
    /// - Returns [`Failure`] with [`ProcessAction::Start`] if the process fails to start
    /// - Returns [`Failure`] with [`ProcessAction::Execute`] if the process exits with non-zero status
    fn run(&mut self) -> impl Future<Output = Result<Output, Failure<ProcessAction>>> + Send;
}

impl ProcessExt for Command {
    async fn run(&mut self) -> Result<Output, Failure<ProcessAction>> {
        let program = self.as_std().get_program().to_string_lossy().to_string();
        let output = self
            .output()
            .await
            .map_err(|e| Failure::new(ProcessAction::Start, e).with("program", &program))?;
        require_success(output, &program)
    }
}

/// Check that a process output indicates success.
///
/// Use this for processes that use `.spawn()` + `.wait_with_output()` patterns
/// where you can't use [`ProcessExt::run()`].
pub fn require_success(output: Output, program: &str) -> Result<Output, Failure<ProcessAction>> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(
            Failure::new(ProcessAction::Execute, ProcessOutput::from(output))
                .with("program", program),
        )
    }
}
