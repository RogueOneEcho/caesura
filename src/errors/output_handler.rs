use crate::errors::command_error::CommandError;
use crate::errors::AppError;
use std::os::unix::process::ExitStatusExt;
use std::process::Output;

pub struct OutputHandler {}

impl OutputHandler {
    pub fn execute(output: Output, action: &str, domain: &str) -> Result<Output, AppError> {
        if output.status.success() {
            Ok(output)
        } else {
            let error = CommandError {
                stderr: String::from_utf8(output.stderr).unwrap_or_default(),
                stdout: String::from_utf8(output.stdout).unwrap_or_default(),
                exit_code: output.status.code(),
                exit_signal: output.status.signal(),
                exit_stopped_signal: output.status.stopped_signal(),
            };
            AppError::command(error, action, domain)
        }
    }
}
