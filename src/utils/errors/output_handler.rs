use rogue_logging::Error;
use std::process::{ExitStatus, Output};

use crate::utils::*;

pub struct OutputHandler {}

impl OutputHandler {
    pub fn execute(output: Output, action: &str, domain: &str) -> Result<Output, Error> {
        if output.status.success() {
            Ok(output)
        } else {
            let error = CommandError {
                stderr: String::from_utf8(output.stderr).unwrap_or_default(),
                stdout: String::from_utf8(output.stdout).unwrap_or_default(),
                exit_code: output.status.code(),
                exit_signal: get_signal(output.status),
                exit_stopped_signal: get_stopped_signal(output.status),
            };
            Err(output_error(error, action, domain))
        }
    }
}

#[cfg(unix)]
fn get_signal(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(windows)]
fn get_signal(_status: ExitStatus) -> Option<i32> {
    None
}

#[cfg(unix)]
fn get_stopped_signal(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.stopped_signal()
}

#[cfg(windows)]
fn get_stopped_signal(_status: ExitStatus) -> Option<i32> {
    None
}
