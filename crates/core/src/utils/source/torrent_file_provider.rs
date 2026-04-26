use crate::prelude::*;
use tokio::io::AsyncWriteExt;

/// Download and cache source `.torrent` files from the Gazelle API.
#[injectable]
pub(crate) struct TorrentFileProvider {
    /// Gazelle API client.
    pub(crate) api: Ref<GazelleClient>,
    /// Path manager for resolving cache paths.
    pub(crate) paths: Ref<PathManager>,
}

impl TorrentFileProvider {
    /// Retrieve a `.torrent` file, downloading from the API if not cached.
    ///
    /// Resolves the cache path via [`PathManager`], returns immediately if
    /// cached. Otherwise downloads the torrent, writes atomically via a
    /// `.tmp` intermediate, and renames into place.
    pub(crate) async fn get(&self, torrent_id: u32) -> Result<PathBuf, Failure<TorrentFileAction>> {
        let path = self.paths.get_source_torrent_path(torrent_id);
        if path.is_file() {
            trace!("{} cached torrent file: {}", "Using".bold(), path.display());
            return Ok(path);
        }
        trace!(
            "{} torrent file as it's not cached: {}",
            "Downloading".bold(),
            path.display()
        );
        let parent = path.parent().expect("torrent path should have parent");
        if !parent.is_dir() {
            create_dir(parent).map_err(Failure::wrap_with_path(
                TorrentFileAction::CreateDirectory,
                parent,
            ))?;
        }
        let mut tmp_path = path.clone();
        tmp_path.as_mut_os_string().push(".tmp");
        let mut file = TokioFile::create(&tmp_path)
            .await
            .map_err(Failure::wrap_with_path(
                TorrentFileAction::CreateFile,
                &tmp_path,
            ))?;
        let buffer = self
            .api
            .download_torrent(torrent_id)
            .await
            .map_err(Failure::wrap(TorrentFileAction::Download))?;
        file.write_all(&buffer)
            .await
            .map_err(Failure::wrap_with_path(
                TorrentFileAction::WriteFile,
                &tmp_path,
            ))?;
        file.flush().await.map_err(Failure::wrap_with_path(
            TorrentFileAction::FlushFile,
            &tmp_path,
        ))?;
        drop(file);
        tokio_rename(&tmp_path, &path)
            .await
            .map_err(Failure::wrap_with(TorrentFileAction::RenameFile, |f| {
                f.with("from", tmp_path.display().to_string())
                    .with("to", path.display().to_string())
            }))?;
        Ok(path)
    }
}

/// Error action for [`TorrentFileProvider`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum TorrentFileAction {
    /// Create the torrent cache directory.
    #[error("create torrent cache directory")]
    CreateDirectory,
    /// Create a torrent file on disk.
    #[error("create torrent file")]
    CreateFile,
    /// Download the torrent from the API.
    #[error("download torrent")]
    Download,
    /// Write torrent data to disk.
    #[error("write torrent file")]
    WriteFile,
    /// Flush torrent file to disk.
    #[error("flush torrent file")]
    FlushFile,
    /// Rename temporary torrent file to final path.
    #[error("rename torrent file")]
    RenameFile,
}
