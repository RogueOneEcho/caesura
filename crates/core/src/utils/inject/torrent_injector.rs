use crate::prelude::*;
use qbittorrent_api::add_torrent::AddTorrentOptions;

/// Reusable helper for injecting torrents into qBittorrent and copying
/// torrent files to disk.
#[injectable]
pub(crate) struct TorrentInjector {
    copy_options: Ref<CopyOptions>,
    qbit: Ref<QbitClient>,
}

impl TorrentInjector {
    /// Inject a `.torrent` file into qBittorrent.
    ///
    /// - Returns `Err(InjectQbit)` if the API call fails or returns a non-`true` result.
    pub(crate) async fn inject_qbit(
        &self,
        torrent_path: &Path,
        add_options: AddTorrentOptions,
    ) -> Result<(), Failure<InjectAction>> {
        let response = self
            .qbit
            .add_torrent(add_options, torrent_path.to_path_buf())
            .await
            .map_err(Failure::wrap_with_path(
                InjectAction::InjectQbit,
                torrent_path,
            ))?;
        if response.result == Some(true) {
            Ok(())
        } else {
            Err(Failure::from_action(InjectAction::InjectQbit)
                .with_path(torrent_path)
                .with("reason", "qBittorrent rejected the torrent"))
        }
    }

    /// Copy (or hard-link, per [`CopyOptions::hard_link`]) a `.torrent` file
    /// into a target directory.
    pub(crate) async fn copy_torrent(
        &self,
        torrent_path: &Path,
        target_dir: &Path,
    ) -> Result<(), Failure<InjectAction>> {
        let file_name = torrent_path
            .file_name()
            .expect("torrent path should have a file name");
        let target_path = target_dir.join(file_name);
        if self.copy_options.hard_link {
            tokio_hard_link(torrent_path, &target_path)
                .await
                .map_err(Failure::wrap_with_path(
                    InjectAction::HardLinkTorrent,
                    &target_path,
                ))?;
            trace!(
                "{} {} to {}",
                "Hard Linked".bold(),
                torrent_path.display(),
                target_path.display()
            );
        } else {
            tokio_copy(torrent_path, &target_path)
                .await
                .map_err(Failure::wrap_with_path(
                    InjectAction::CopyTorrent,
                    &target_path,
                ))?;
            trace!(
                "{} {} to {}",
                "Copied".bold(),
                torrent_path.display(),
                target_path.display()
            );
        }
        Ok(())
    }
}

/// Error action for [`TorrentInjector`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum InjectAction {
    /// Inject a torrent into qBittorrent via the API.
    #[error("inject torrent into qBittorrent")]
    InjectQbit,
    /// Copy a torrent file on disk.
    #[error("copy torrent file")]
    CopyTorrent,
    /// Hard-link a torrent file on disk.
    #[error("hard link torrent file")]
    HardLinkTorrent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;
    use qbittorrent_api::Response;
    use std::fs::create_dir_all;

    const FAKE_TORRENT: &[u8] = b"fake-torrent";

    /// Verify `inject_qbit` succeeds when the mock client returns `result: true`.
    #[tokio::test]
    async fn torrent_injector_inject_qbit_success() {
        // Arrange
        let dir = TempDirectory::create("torrent_injector_inject_qbit_success");
        let torrent_path = write_fake_torrent(&dir);
        let qbit = MockQBittorrentClient::new().with_add_torrents(Response {
            status_code: Some(200),
            result: Some(true),
        });
        let injector = make_injector(qbit, CopyOptions::default());

        // Act
        let result = injector
            .inject_qbit(&torrent_path, AddTorrentOptions::default())
            .await;

        // Assert
        assert!(result.is_ok(), "inject should succeed: {result:?}");
    }

    /// Verify `inject_qbit` errors when the mock client returns `result: false`.
    #[tokio::test]
    async fn torrent_injector_inject_qbit_rejected() {
        // Arrange
        let dir = TempDirectory::create("torrent_injector_inject_qbit_rejected");
        let torrent_path = write_fake_torrent(&dir);
        let qbit = MockQBittorrentClient::new().with_add_torrents(Response {
            status_code: Some(200),
            result: Some(false),
        });
        let injector = make_injector(qbit, CopyOptions::default());

        // Act
        let result = injector
            .inject_qbit(&torrent_path, AddTorrentOptions::default())
            .await;

        // Assert
        assert!(result.is_err(), "inject should fail when result is false");
    }

    /// Verify `copy_torrent` copies the file when `hard_link` is false.
    #[tokio::test]
    async fn torrent_injector_copy_torrent_copies() {
        // Arrange
        let dir = TempDirectory::create("torrent_injector_copy_torrent_copies");
        let torrent_path = write_fake_torrent(&dir);
        let target_dir = dir.join("watch");
        create_dir_all(&target_dir).expect("create target dir");
        let injector = make_injector(
            MockQBittorrentClient::default(),
            CopyOptions { hard_link: false },
        );

        // Act
        let result = injector.copy_torrent(&torrent_path, &target_dir).await;

        // Assert
        assert!(result.is_ok(), "copy should succeed: {result:?}");
        let copied = target_dir.join("source.torrent");
        assert!(copied.exists(), "copied file should exist at {copied:?}");
        let bytes = read(&copied).expect("read copied file");
        assert_eq!(bytes, FAKE_TORRENT);
    }

    /// Verify `copy_torrent` hard-links the file when `hard_link` is true.
    #[tokio::test]
    async fn torrent_injector_copy_torrent_hard_links() {
        // Arrange
        let dir = TempDirectory::create("torrent_injector_copy_torrent_hard_links");
        let torrent_path = write_fake_torrent(&dir);
        let target_dir = dir.join("watch");
        create_dir_all(&target_dir).expect("create target dir");
        let injector = make_injector(
            MockQBittorrentClient::default(),
            CopyOptions { hard_link: true },
        );

        // Act
        let result = injector.copy_torrent(&torrent_path, &target_dir).await;

        // Assert
        assert!(result.is_ok(), "hard link should succeed: {result:?}");
        let linked = target_dir.join("source.torrent");
        assert!(linked.exists(), "linked file should exist at {linked:?}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let source_meta = metadata(&torrent_path).expect("source meta");
            let linked_meta = metadata(&linked).expect("linked meta");
            assert_eq!(
                source_meta.ino(),
                linked_meta.ino(),
                "hard link should share inode"
            );
        }
    }

    /// Write a fake `source.torrent` file into `dir` and return its path.
    fn write_fake_torrent(dir: &Path) -> PathBuf {
        let path = dir.join("source.torrent");
        write(&path, FAKE_TORRENT).expect("write torrent");
        path
    }

    #[expect(
        clippy::as_conversions,
        reason = "required for trait object boxing in test"
    )]
    fn make_injector(qbit: MockQBittorrentClient, copy_options: CopyOptions) -> TorrentInjector {
        let qbit: Ref<QbitClient> = Ref::new(Box::new(qbit) as QbitClient);
        TorrentInjector {
            copy_options: Ref::new(copy_options),
            qbit,
        }
    }
}
