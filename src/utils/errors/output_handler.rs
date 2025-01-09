use rogue_logging::Error;
use std::os::unix::process::ExitStatusExt;
use std::process::Output;

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
                exit_signal: output.status.signal(),
                exit_stopped_signal: output.status.stopped_signal(),
            };
            Err(output_error(error, action, domain))
        }
    }
}
