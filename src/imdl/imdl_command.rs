use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

use crate::dependencies::IMDL;
use bytes::Buf;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::{AppError, OutputHandler};
use crate::imdl::TorrentSummary;
use crate::verify::SourceRule;
use crate::verify::SourceRule::IncorrectHash;

pub struct ImdlCommand;

impl ImdlCommand {
    /// Create a torrent
    pub async fn create(
        content_dir: &Path,
        output_path: &Path,
        announce_url: String,
        source: String,
    ) -> Result<Output, AppError> {
        let action = "create torrent";
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("create")
            .arg(content_dir.to_string_lossy().to_string())
            .arg("--private")
            .arg("--announce")
            .arg(announce_url)
            .arg("--source")
            .arg(source)
            .arg("--output")
            .arg(output_path.to_string_lossy().to_string())
            .arg("--force")
            .output()
            .await
            .or_else(|e| AppError::io(e, action))?;
        OutputHandler::execute(output, action, "IMDL")
    }

    /// Get a summary of the torrent file.
    pub async fn show(path: &Path) -> Result<TorrentSummary, AppError> {
        let action = "read torrent";
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("show")
            .arg("--json")
            .arg(path)
            .output()
            .await
            .or_else(|e| AppError::io(e, action))?;
        let output = OutputHandler::execute(output, action, "IMDL")?;
        let reader = output.stdout.reader();
        serde_json::from_reader(reader).or_else(|e| AppError::deserialization(e, action))
    }

    /// Verify files match the torrent metadata.
    pub async fn verify(buffer: &[u8], directory: &PathBuf) -> Result<Vec<SourceRule>, AppError> {
        let action = "verify torrent";
        let mut child = Command::new(IMDL)
            .arg("torrent")
            .arg("verify")
            .arg("--content")
            .arg(directory)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .or_else(|e| AppError::io(e, action))?;
        let mut stdin = child.stdin.take().expect("stdin should be available");
        stdin
            .write_all(buffer)
            .await
            .or_else(|e| AppError::io(e, action))?;
        drop(stdin);
        let output = child
            .wait_with_output()
            .await
            .or_else(|e| AppError::io(e, action))?;
        if output.status.success() {
            Ok(Vec::new())
        } else {
            let details = String::from_utf8(output.stderr).unwrap_or_default();
            Ok(vec![IncorrectHash(details)])
        }
    }
}
