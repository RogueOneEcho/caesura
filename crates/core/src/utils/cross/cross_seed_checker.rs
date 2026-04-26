use crate::prelude::*;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::Torrent as LavaTorrent;

const EXTENSIONS: &[&str] = &[".flac", ".mp3"];

/// Checks whether a source torrent exists on the cross indexer.
///
/// Uses hash-based lookup first (source-field swap), then falls back to a
/// filename browse with size, file count, file path, and fileset verification.
pub(crate) struct CrossSeedChecker {
    /// Gazelle client for the cross indexer. `None` when cross indexer is not configured.
    pub(crate) api: Ref<GazelleClient>,
    /// Main indexer used to decode the `release_type` ID on the source torrent.
    pub(crate) main: Indexer,
    /// Cross indexer used to decode the `release_type` ID on the source torrent.
    pub(crate) cross: Indexer,
}

impl CrossSeedChecker {
    /// Find the torrent ID on the cross indexer matching the given source, if any.
    ///
    /// - Callers must ensure [`Self::is_configured`] returns `true`.
    pub(crate) async fn execute(
        &self,
        torrent_path: &Path,
        source: &Source,
    ) -> Result<Option<u32>, Failure<CrossSeedAction>> {
        let torrent = LavaTorrent::read_from_file(torrent_path).map_err(
            Failure::wrap_with_path(CrossSeedAction::ReadTorrent, torrent_path),
        )?;
        if let Some(id) = self.check_by_hash(&torrent, source).await? {
            return Ok(Some(id));
        }
        self.check_by_filelist(source).await
    }

    /// Look up a torrent on the cross indexer by its re-sourced info hash.
    ///
    /// - `Ok(Some(id))` - cross indexer has a torrent with the swapped hash
    /// - `Ok(None)` - cross indexer returned `BadRequest` or `NotFound`
    /// - `Err(_)` - any other API failure
    async fn check_by_hash(
        &self,
        torrent: &LavaTorrent,
        source: &Source,
    ) -> Result<Option<u32>, Failure<CrossSeedAction>> {
        let hash = compute_cross_hash(torrent, &self.cross.to_uppercase());
        match self.api.get_torrent_by_hash(&hash).await {
            Ok(response) => {
                debug!(
                    "{} cross indexer match by hash for torrent {}",
                    "Found".bold(),
                    source.torrent.id
                );
                Ok(Some(response.torrent.id))
            }
            Err(error) if error.is_missing() => Ok(None),
            Err(error) => Err(Failure::new(CrossSeedAction::HashLookup, error)
                .with("torrent_id", source.torrent.id.to_string())),
        }
    }

    async fn check_by_filelist(
        &self,
        source: &Source,
    ) -> Result<Option<u32>, Failure<CrossSeedAction>> {
        let source_files = source.torrent.get_files();
        let Some(filename) = select_search_filename(&source_files) else {
            return Ok(None);
        };
        let results = self.get_candidates(source, filename).await?;
        for group in &results.results {
            for candidate in &group.torrents {
                if !passes_prefilter(candidate, source.torrent.size, source.torrent.file_count) {
                    continue;
                }
                let response = self.api.get_torrent(candidate.torrent_id).await.map_err(
                    Failure::wrap_with(CrossSeedAction::GetTorrent, |f| {
                        f.with("torrent_id", candidate.torrent_id.to_string())
                    }),
                )?;
                if response.torrent.file_path != source.torrent.file_path {
                    continue;
                }
                if response.torrent.encoding != source.torrent.encoding {
                    continue;
                }
                if response.torrent.get_files() == source_files {
                    debug!(
                        "{} cross indexer match by filelist for torrent {}",
                        "Found".bold(),
                        source.torrent.id
                    );
                    return Ok(Some(response.torrent.id));
                }
            }
        }
        Ok(None)
    }

    async fn get_candidates(
        &self,
        source: &Source,
        filename: String,
    ) -> Result<BrowseResponse, Failure<CrossSeedAction>> {
        let cross_release_type_id = self.convert_release_type(source.group.release_type);
        let request = BrowseRequest {
            category: Some(Category::Music),
            filelist: Some(filename),
            media: Some(source.torrent.media.clone()),
            release_type: cross_release_type_id,
            ..BrowseRequest::default()
        };
        let results = self
            .api
            .browse(&request)
            .await
            .map_err(Failure::wrap(CrossSeedAction::Browse))?;
        Ok(results)
    }

    /// Convert the source torrent's `release_type` ID from main-indexer numbering
    /// to cross-indexer numbering.
    ///
    /// Returns `None` if the ID is unknown on the main indexer or has no
    /// equivalent on the cross indexer; callers omit the filter in that case.
    fn convert_release_type(&self, id: ReleaseTypeId) -> Option<ReleaseTypeId> {
        let release_type = match self.main {
            Indexer::Ops => ReleaseType::from_int_ops(id)?,
            _ => ReleaseType::from_int_red(id)?,
        };
        match self.cross {
            Indexer::Ops => release_type.to_id_ops(),
            _ => release_type.to_id_red(),
        }
    }
}

/// Return `true` if a browse result passes the pre-filters before a full API fetch.
pub(crate) fn passes_prefilter(
    candidate: &BrowseTorrent,
    source_size: u64,
    source_file_count: u32,
) -> bool {
    candidate.size == source_size && candidate.file_count == source_file_count
}

/// Select the best filename from the file list to use as a browse search query.
///
/// - Only considers music files (FLAC, MP3, etc.)
/// - Picks the longest name (most distinctive)
/// - Returns the full filename including extension
fn select_search_filename(files: &[FileItem]) -> Option<String> {
    files
        .iter()
        .filter(|item| {
            let lower = item.name.to_lowercase();
            EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
        })
        .max_by_key(|item| item.name.len())
        .map(|item| item.name.clone())
}

/// Compute the info hash this torrent would have with its `source` field swapped.
fn compute_cross_hash(torrent: &LavaTorrent, target_source: &str) -> String {
    let mut torrent = torrent.clone();
    let fields = torrent.extra_info_fields.get_or_insert_with(HashMap::new);
    fields.insert(
        "source".to_owned(),
        BencodeElem::String(target_source.to_owned()),
    );
    torrent.info_hash()
}

/// Error action for [`CrossSeedChecker`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum CrossSeedAction {
    /// Read and parse the source `.torrent` file from disk.
    #[error("read source torrent file")]
    ReadTorrent,
    /// Look up the cross torrent by info hash.
    #[error("look up torrent by hash on cross indexer")]
    HashLookup,
    /// Browse the cross indexer for candidates.
    #[error("browse cross indexer")]
    Browse,
    /// Fetch full torrent details from the cross indexer.
    #[error("get torrent from cross indexer")]
    GetTorrent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gazelle_api::{
        ApiResponseError, BrowseGroup, ErrorSource, GazelleError, MockGazelleClient,
        TorrentResponse,
    };
    use lava_torrent::torrent::v1::TorrentBuilder;
    use std::fs::write as fs_write;

    /// Verify the pre-filter accepts candidates with matching size and file count.
    #[test]
    fn passes_prefilter_matching_candidate() {
        let candidate = BrowseTorrent {
            size: 1_000,
            file_count: 10,
            ..BrowseTorrent::mock()
        };
        assert!(passes_prefilter(&candidate, 1_000, 10));
    }

    /// Verify candidates with wrong size are rejected.
    #[test]
    fn passes_prefilter_rejects_wrong_size() {
        let candidate = BrowseTorrent {
            size: 999,
            file_count: 10,
            ..BrowseTorrent::mock()
        };
        assert!(!passes_prefilter(&candidate, 1_000, 10));
    }

    /// Verify candidates with wrong file count are rejected.
    #[test]
    fn passes_prefilter_rejects_wrong_file_count() {
        let candidate = BrowseTorrent {
            size: 1_000,
            file_count: 9,
            ..BrowseTorrent::mock()
        };
        assert!(!passes_prefilter(&candidate, 1_000, 10));
    }

    /// Verify the longest music filename is selected.
    #[test]
    fn select_search_filename_picks_longest_music_file() {
        // Arrange
        let files = vec![
            FileItem {
                name: "short.flac".to_owned(),
                size: 100,
            },
            FileItem {
                name: "a very long track name indeed.flac".to_owned(),
                size: 200,
            },
            FileItem {
                name: "cover.jpg".to_owned(),
                size: 50,
            },
        ];

        // Act
        let output = select_search_filename(&files);

        // Assert
        assert_eq!(
            output,
            Some("a very long track name indeed.flac".to_owned())
        );
    }

    /// Verify `None` is returned when no music files are present.
    #[test]
    fn select_search_filename_no_music_returns_none() {
        let files = vec![
            FileItem {
                name: "cover.jpg".to_owned(),
                size: 50,
            },
            FileItem {
                name: "booklet.pdf".to_owned(),
                size: 1000,
            },
        ];
        assert!(select_search_filename(&files).is_none());
    }

    /// Verify swapping the source field changes the computed hash.
    #[test]
    fn compute_cross_hash_differs_when_source_changes() {
        // Arrange
        let dir = TempDirectory::create("compute_cross_hash_differs_when_source_changes");
        fs_write(dir.join("track.flac"), b"x").expect("write");
        let torrent = TorrentBuilder::new(&*dir, 16384)
            .set_announce(Some("https://example.com/announce".to_owned()))
            .set_privacy(true)
            .add_extra_info_field("source".to_owned(), BencodeElem::String("RED".to_owned()))
            .build()
            .expect("build");

        // Act
        let red_hash = compute_cross_hash(&torrent, "RED");
        let ops_hash = compute_cross_hash(&torrent, "OPS");

        // Assert
        assert_ne!(red_hash, ops_hash);
    }

    /// Verify `release_type` conversion from RED ID to OPS ID.
    #[test]
    fn cross_seed_checker_convert_release_type_red_to_ops() {
        // Arrange
        let checker = CrossSeedChecker {
            api: mock_api(MockGazelleClient::new()),
            main: Indexer::Red,
            cross: Indexer::Ops,
        };

        // Act
        let output = checker.convert_release_type(ReleaseTypeId::from_int(1));

        // Assert
        assert!(output.is_some());
        let red_album =
            ReleaseType::from_int_red(ReleaseTypeId::from_int(1)).expect("RED ID 1 is Album");
        assert_eq!(output, red_album.to_id_ops());
    }

    /// Verify an unknown `release_type` ID returns `None` without panicking.
    #[test]
    fn cross_seed_checker_convert_release_type_unknown() {
        let checker = CrossSeedChecker {
            api: mock_api(MockGazelleClient::new()),
            main: Indexer::Red,
            cross: Indexer::Ops,
        };
        assert!(
            checker
                .convert_release_type(ReleaseTypeId::from_int(9999))
                .is_none()
        );
    }

    /// Verify `execute()` returns the cross torrent ID when hash lookup succeeds.
    #[tokio::test]
    async fn cross_seed_checker_execute_hash_match() {
        // Arrange
        let mut expected = TorrentResponse::mock();
        expected.torrent.id = 987_654;
        let mock = MockGazelleClient::new().with_get_torrent_by_hash(Ok(expected.clone()));
        let checker = CrossSeedChecker {
            api: mock_api(mock),
            main: Indexer::Red,
            cross: Indexer::Ops,
        };
        let source = Source::mock();
        let dir = TempDirectory::create("cross_seed_checker_execute_hash_match");
        fs_write(dir.join("t.flac"), b"x").expect("write");
        let torrent_path = dir.join("source.torrent");
        let torrent_bytes = TorrentBuilder::new(&*dir, 16384)
            .set_announce(Some("https://example.com/announce".to_owned()))
            .set_privacy(true)
            .add_extra_info_field("source".to_owned(), BencodeElem::String("RED".to_owned()))
            .build()
            .expect("build")
            .encode()
            .expect("encode");
        fs_write(&torrent_path, &torrent_bytes).expect("write torrent file");

        // Act
        let output = checker
            .execute(&torrent_path, &source)
            .await
            .expect("execute");

        // Assert
        assert_eq!(output, Some(987_654));
    }

    /// Verify filelist fallback returns the candidate id when fileset matches.
    #[tokio::test]
    async fn cross_seed_checker_execute_filelist_match() {
        // Arrange
        let source = Source::mock();
        let browse = BrowseResponse {
            results: vec![BrowseGroup {
                torrents: vec![BrowseTorrent {
                    torrent_id: 555,
                    size: source.torrent.size,
                    file_count: source.torrent.file_count,
                    ..BrowseTorrent::mock()
                }],
                ..BrowseGroup::mock()
            }],
            ..BrowseResponse::mock()
        };
        let matching = TorrentResponse {
            torrent: Torrent {
                id: 555,
                file_path: source.torrent.file_path.clone(),
                file_list: source.torrent.file_list.clone(),
                encoding: source.torrent.encoding.clone(),
                ..Torrent::mock()
            },
            ..TorrentResponse::mock()
        };
        let mock = MockGazelleClient::new()
            .with_get_torrent_by_hash(Err(not_found_error()))
            .with_browse(Ok(browse))
            .with_get_torrent(Ok(matching));
        let checker = CrossSeedChecker {
            api: mock_api(mock),
            main: Indexer::Red,
            cross: Indexer::Ops,
        };
        let torrent_path = write_mock_torrent();

        // Act
        let output = checker
            .execute(torrent_path.path(), &source)
            .await
            .expect("execute");

        // Assert
        assert_eq!(output, Some(555));
    }

    /// Verify filelist fallback returns `None` when candidate fileset differs.
    #[tokio::test]
    async fn cross_seed_checker_execute_filelist_mismatch() {
        // Arrange
        let source = Source::mock();
        let browse = BrowseResponse {
            results: vec![BrowseGroup {
                torrents: vec![BrowseTorrent {
                    torrent_id: 555,
                    size: source.torrent.size,
                    file_count: source.torrent.file_count,
                    ..BrowseTorrent::mock()
                }],
                ..BrowseGroup::mock()
            }],
            ..BrowseResponse::mock()
        };
        let differing = TorrentResponse {
            torrent: Torrent {
                id: 555,
                file_path: source.torrent.file_path.clone(),
                file_list: "different.flac{{{999999}}}|||".to_owned(),
                encoding: source.torrent.encoding.clone(),
                ..Torrent::mock()
            },
            ..TorrentResponse::mock()
        };
        let mock = MockGazelleClient::new()
            .with_get_torrent_by_hash(Err(not_found_error()))
            .with_browse(Ok(browse))
            .with_get_torrent(Ok(differing));
        let checker = CrossSeedChecker {
            api: mock_api(mock),
            main: Indexer::Red,
            cross: Indexer::Ops,
        };
        let torrent_path = write_mock_torrent();

        // Act
        let output = checker
            .execute(torrent_path.path(), &source)
            .await
            .expect("execute");

        // Assert
        assert_eq!(output, None);
    }

    #[expect(
        clippy::as_conversions,
        reason = "required for trait object boxing in test"
    )]
    fn mock_api(mock: MockGazelleClient) -> Ref<GazelleClient> {
        Ref::new(Box::new(mock) as GazelleClient)
    }

    fn not_found_error() -> GazelleError {
        GazelleError {
            operation: GazelleOperation::ApiResponse(ApiResponseKind::NotFound),
            source: ErrorSource::ApiResponse(ApiResponseError {
                message: "not found".to_owned(),
                status: 404,
            }),
        }
    }

    /// Build a minimal on-disk `.torrent` file for [`CrossSeedChecker::execute`].
    ///
    /// - Returns the owning [`TempDirectory`] so the caller keeps the path alive.
    fn write_mock_torrent() -> MockTorrentFile {
        let dir = TempDirectory::create("write_mock_torrent");
        fs_write(dir.join("t.flac"), b"x").expect("write");
        let torrent_path = dir.join("source.torrent");
        let torrent_bytes = TorrentBuilder::new(&*dir, 16384)
            .set_announce(Some("https://example.com/announce".to_owned()))
            .set_privacy(true)
            .add_extra_info_field("source".to_owned(), BencodeElem::String("RED".to_owned()))
            .build()
            .expect("build")
            .encode()
            .expect("encode");
        fs_write(&torrent_path, &torrent_bytes).expect("write torrent file");
        MockTorrentFile { dir, torrent_path }
    }

    struct MockTorrentFile {
        #[expect(dead_code, reason = "kept to retain ownership of the temp directory")]
        dir: TempDirectory,
        torrent_path: PathBuf,
    }

    impl MockTorrentFile {
        fn path(&self) -> &Path {
            &self.torrent_path
        }
    }
}
