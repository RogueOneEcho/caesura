use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};

use crate::imdl::ImdlError;
use crate::imdl::ImdlError::*;
use bytes::Buf;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::imdl::torrent_summary::TorrentSummary;

/// Path to the imdl binary
#[cfg(target_os = "windows")]
const BINARY_PATH: &str = "imdl.exe";

/// Path to the imdl binary.
#[cfg(not(target_os = "windows"))]
const BINARY_PATH: &str = "imdl";

pub struct ImdlCommand;

impl ImdlCommand {
    /// Create a torrent
    pub async fn create(
        content_path: &Path,
        torrent_path: &Path,
        announce_url: String,
    ) -> Result<ExitStatus, std::io::Error> {
        let mut child = Command::new(BINARY_PATH)
            .arg("torrent")
            .arg("create")
            .arg(content_path.to_string_lossy().to_string())
            .arg("-P")
            .arg("-a")
            .arg(announce_url)
            .arg("-s")
            .arg("RED")
            .arg("-o")
            .arg(torrent_path.to_string_lossy().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        child.wait().await
    }

    /// Get a summary of the torrent file.
    pub async fn show(path: &Path) -> Result<TorrentSummary, ImdlError> {
        let result = Command::new(BINARY_PATH)
            .arg("torrent")
            .arg("show")
            .arg("--json")
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;
        let output = match result {
            Ok(output) => output,
            Err(error) => return Err(IOFailure(error)),
        };
        let reader = output.stdout.reader();
        match serde_json::from_reader(reader) {
            Ok(summary) => Ok(summary),
            Err(error) => Err(DeserializationFailure(error)),
        }
    }

    /// Verify files match the torrent metadata.
    pub async fn verify(buffer: &[u8], directory: &PathBuf) -> Result<bool, std::io::Error> {
        let mut child = Command::new(BINARY_PATH)
            .arg("torrent")
            .arg("verify")
            .arg("--content")
            .arg(directory)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child.stdin.take().expect("stdin should be available");
        stdin.write_all(buffer).await?;
        drop(stdin);
        let output = child.wait_with_output().await?;
        // TODO SHOULD retrieve explanation of invalid files from stderr
        Ok(output.status.success())
    }
}
