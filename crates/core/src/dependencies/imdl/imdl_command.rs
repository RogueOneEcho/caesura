use crate::built_info::{PKG_NAME, PKG_VERSION};
use crate::prelude::*;
use crate::utils::SourceIssue::Imdl;
use bytes::Buf;
use std::process::Output;
use tokio::fs::copy;
use tokio::process::Command;

/// Facade for the `imdl` CLI binary.
///
/// Invokes `imdl` as a subprocess for torrent creation and verification.
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
        Command::new(IMDL)
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
            .run()
            .await
            .map_err(|e| process_error(e, "create torrent", IMDL))
    }

    /// Get a summary of the torrent file.
    pub async fn show(path: &Path) -> Result<TorrentSummary, Error> {
        let output = Command::new(IMDL)
            .arg("torrent")
            .arg("show")
            .arg("--json")
            .arg(path)
            .run()
            .await
            .map_err(|e| process_error(e, "read torrent", IMDL))?;
        let reader = output.stdout.reader();
        serde_json::from_reader(reader).map_err(|e| json_error(e, "deserialize torrent"))
    }

    /// Verify files match the torrent metadata.
    pub async fn verify(
        torrent_file: &Path,
        directory: &Path,
    ) -> Result<Option<SourceIssue>, Error> {
        match Command::new(IMDL)
            .arg("torrent")
            .arg("verify")
            .arg("--content")
            .arg(directory)
            .arg(torrent_file)
            .run()
            .await
        {
            Ok(_) => Ok(None),
            Err(ProcessError::Failed(output)) => {
                let details = output.stderr.unwrap_or_default();
                Ok(Some(Imdl { details }))
            }
            Err(e) => Err(process_error(e, "verify torrent", IMDL)),
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

    use super::*;
    use crate::utils::SAMPLE_SOURCES_DIR;

    #[tokio::test]
    async fn imdl_show() -> Result<(), Error> {
        // Arrange
        let album = AlbumProvider::get(SampleFormat::default()).await;
        let path = SAMPLE_SOURCES_DIR.join(album.torrent_filename());

        // Act
        let summary = ImdlCommand::show(&path).await?;

        // Assert
        assert!(!summary.files.is_empty());
        Ok(())
    }
}
