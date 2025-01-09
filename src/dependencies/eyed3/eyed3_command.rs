use rogue_logging::Error;
use std::path::Path;
use tokio::process::Command;

use crate::dependencies::*;
use crate::utils::*;

pub struct EyeD3Command;

impl EyeD3Command {
    /// Create a torrent
    pub async fn display(path: &Path) -> Result<String, Error> {
        let output = Command::new(EYED3)
            .arg(path.to_string_lossy().to_string())
            .arg("--no-color")
            .arg("-r")
            .output()
            .await
            .map_err(|e| command_error(e, "get details", EYED3))?;
        let output = OutputHandler::execute(output, "get details", "eyeD3")?;
        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    }
}
