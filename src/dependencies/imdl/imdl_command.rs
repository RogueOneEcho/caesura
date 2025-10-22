use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

use crate::built_info::{PKG_NAME, PKG_VERSION};
use crate::dependencies::*;
use crate::utils::SourceIssue::Imdl;
use crate::utils::*;
use bytes::Buf;
use colored::Colorize;
use log::trace;
use rogue_logging::Error;
use tokio::fs::copy;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct ImdlCommand;

impl ImdlCommand {
    #[allow(clippy::uninlined_format_args)]
    /// Create a torrent
    pub async fn create(
        content_dir: &Path,
        output_path: &Path,
        announce_url: String,
        source: String,
    ) -> Result<Output, Error> {
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("create")
            .arg(content_dir.to_string_lossy().to_string())
            .arg("--private")
            .arg("--announce")
            .arg(announce_url)
            .arg("--comment")
            .arg(format!("Created with {} v{}", PKG_NAME, PKG_VERSION))
            .arg("--source")
            .arg(source.to_uppercase())
            .arg("--output")
            .arg(output_path.to_string_lossy().to_string())
            .arg("--no-created-by")
            .arg("--force")
            .output()
            .await
            .map_err(|e| command_error(e, "execute create torrent", IMDL))?;
        OutputHandler::execute(output, "create torrent", "IMDL")
    }

    /// Get a summary of the torrent file.
    pub async fn show(path: &Path) -> Result<TorrentSummary, Error> {
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("show")
            .arg("--json")
            .arg(path)
            .output()
            .await
            .map_err(|e| command_error(e, "execute read torrent", IMDL))?;
        let output = OutputHandler::execute(output, "read torrent", "IMDL")?;
        let reader = output.stdout.reader();
        serde_json::from_reader(reader).map_err(|e| json_error(e, "deserialize torrent"))
    }

    /// Verify files match the torrent metadata.
    pub async fn verify(
        torrent_file: &Path,
        directory: &Path,
    ) -> Result<Option<SourceIssue>, Error> {
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("verify")
            .arg("--content")
            .arg(directory)
            .arg(torrent_file)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| command_error(e, "execute verify torrent", IMDL))?;
        if output.status.success() {
            Ok(None)
        } else {
            let details = String::from_utf8(output.stderr).unwrap_or_default();
            Ok(Some(Imdl { details }))
        }
    }

    /// Verify files match the torrent metadata.
    #[allow(dead_code)]
    pub async fn verify_from_buffer(
        buffer: &[u8],
        directory: &PathBuf,
    ) -> Result<Vec<SourceIssue>, Error> {
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
            .map_err(|e| command_error(e, "execute verify torrent", IMDL))?;
        let mut stdin = child.stdin.take().expect("stdin should be available");
        stdin
            .write_all(buffer)
            .await
            .map_err(|e| command_error(e, "writing buffer to verify torrent", IMDL))?;
        drop(stdin);
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| command_error(e, "get output of verify torrent", IMDL))?;
        if output.status.success() {
            Ok(Vec::new())
        } else {
            let details = String::from_utf8(output.stderr).unwrap_or_default();
            Ok(vec![Imdl { details }])
        }
    }

    /// Duplicate a .torrent file
    ///
    /// Copy if the source and announce are the same.
    ///
    /// Otherwise, verify content is unchanged and re-create with new source.
    pub async fn duplicate_torrent(
        from: &Path,
        to: &Path,
        content_dir: &Path,
        announce_url: String,
        source: String,
    ) -> Result<bool, Error> {
        let torrent = ImdlCommand::show(from).await?;
        let torrent_announce = torrent.announce_list.first().and_then(|x| x.first());
        if torrent.is_source_equal(&source) && torrent_announce == Some(&announce_url) {
            trace!(
                "{} {:?} to {:?}",
                "Copying".bold(),
                from.file_name(),
                to.file_name()
            );
            copy(&from, &to)
                .await
                .map_err(|e| io_error(e, "duplicate torrent"))?;
            return Ok(true);
        }
        if !content_dir.is_dir() {
            trace!(
                "Torrent content directory does not exist: {}",
                content_dir.display()
            );
            return Ok(false);
        }
        let verify_issues = ImdlCommand::verify(from, content_dir).await?;
        if verify_issues.is_some() {
            trace!(
                "Torrent content failed verification: {:?}",
                from.file_name()
            );
            return Ok(false);
        }
        ImdlCommand::create(content_dir, to, announce_url, source).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use rogue_logging::Error;
    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    #[ignore = "sample required"]
    async fn imdl_show() -> Result<(), Error> {
        // Arrange
        let paths = DirectoryReader::new()
            .with_extension("torrent")
            .read(&PathBuf::from(TORRENTS_SAMPLES_DIR))
            .expect("Directory should exist");
        let path = paths.first().expect("Should be at least one sample");

        // Act
        let summary = ImdlCommand::show(path).await?;

        // Assert
        assert!(!summary.files.is_empty());

        Ok(())
    }
}
