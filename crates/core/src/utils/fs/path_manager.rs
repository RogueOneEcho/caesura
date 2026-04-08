use crate::prelude::*;

/// Known indexers considered when looking for a fallback torrent from another tracker.
const KNOWN_INDEXERS: [Indexer; 3] = [Indexer::Red, Indexer::Ops, Indexer::Pth];

/// Resolve cache, output, and torrent paths from options.
#[injectable]
pub struct PathManager {
    shared_options: Ref<SharedOptions>,
    cache_options: Ref<CacheOptions>,
    file_options: Ref<FileOptions>,
}

impl PathManager {
    /// Default user config file.
    ///
    /// - Docker: `/config.yml`
    /// - Native: platform user config directory
    #[must_use]
    pub fn default_config_path() -> PathBuf {
        if is_docker() {
            return PathBuf::from("/config.yml");
        }
        dirs::config_dir()
            .expect("config directory should be determinable")
            .join(APP_NAME)
            .join("config.yml")
    }

    /// Default user cache directory.
    ///
    /// - Docker: `/cache`
    /// - Native: platform user cache directory
    #[must_use]
    pub fn default_cache_dir() -> PathBuf {
        if is_docker() {
            return PathBuf::from("/cache");
        }
        dirs::cache_dir()
            .expect("cache directory should be determinable")
            .join(APP_NAME)
    }

    /// Default output directory.
    ///
    /// - Docker: `/output`
    /// - Native: platform user data directory
    #[must_use]
    pub fn default_output_dir() -> PathBuf {
        if is_docker() {
            return PathBuf::from("/output");
        }
        dirs::data_dir()
            .expect("data directory should be determinable")
            .join(APP_NAME)
            .join("output")
    }

    /// Cache directory path.
    #[must_use]
    pub fn get_cache_dir(&self) -> PathBuf {
        self.cache_options.path()
    }

    /// Path to the cached source `.torrent` file.
    #[must_use]
    pub fn get_source_torrent_path(&self, source: &Source) -> PathBuf {
        let id = source.torrent.id;
        let indexer = self.shared_options.get_indexer();
        self.get_cache_dir()
            .join("torrents")
            .join(format!("{id}.{}.torrent", indexer.as_lowercase()))
    }

    /// Output directory path with tilde expansion applied.
    #[must_use]
    pub fn get_output_dir(&self) -> PathBuf {
        self.shared_options.output_path()
    }

    /// Output directory for spectrogram images of a source.
    #[must_use]
    pub fn get_spectrogram_dir(&self, source: &Source) -> PathBuf {
        self.get_output_dir()
            .join(SpectrogramName::get(&source.metadata))
    }

    /// Output directory for a transcoded source in the given format.
    #[must_use]
    pub fn get_transcode_target_dir(&self, source: &Source, target: TargetFormat) -> PathBuf {
        self.get_output_dir()
            .join(TranscodeName::get(&source.metadata, target))
    }

    /// Output path for a single transcoded track.
    #[must_use]
    pub fn get_transcode_path(
        &self,
        source: &Source,
        target: TargetFormat,
        flac: &FlacFile,
    ) -> PathBuf {
        let extension = target.get_file_extension();
        let rename_tracks = self.file_options.rename_tracks;
        // If rename_tracks enabled and disc context is set on flac, use renamed paths
        let (base_name, sub_dir) = if rename_tracks && flac.disc_context.is_some() {
            (
                flac.renamed_file_stem(),
                flac.renamed_sub_dir().unwrap_or_default(),
            )
        } else {
            (flac.file_name.clone(), flac.sub_dir.clone())
        };
        self.get_transcode_target_dir(source, target)
            .join(sub_dir)
            .join(format!("{base_name}.{extension}"))
    }

    /// Torrent file path for a transcoded source in the given format.
    #[must_use]
    pub fn get_torrent_path(&self, source: &Source, target: TargetFormat) -> PathBuf {
        let indexer = self.shared_options.get_indexer();
        self.get_torrent_path_for_indexer(source, target, indexer)
    }

    #[must_use]
    fn get_torrent_path_for_indexer(
        &self,
        source: &Source,
        target: TargetFormat,
        indexer: Indexer,
    ) -> PathBuf {
        let mut filename = TranscodeName::get(&source.metadata, target);
        filename.push('.');
        filename.push_str(indexer.as_lowercase());
        filename.push_str(".torrent");
        self.get_output_dir().join(filename)
    }

    /// Torrent path for the current indexer, duplicating from another tracker if needed.
    ///
    /// - Returns the existing path if already present for the current indexer
    /// - Otherwise duplicates from another tracker's torrent file (e.g. `.ops.torrent`)
    /// - Returns `None` if no source torrent is available to duplicate
    ///
    /// Example: `path/to/Artist - Album [2012] [WEB FLAC].red.torrent`
    pub async fn get_or_duplicate_existing_torrent_path(
        &self,
        source: &Source,
        target: TargetFormat,
    ) -> Result<Option<PathBuf>, Failure<TorrentCreateAction>> {
        let target_path = self.get_torrent_path(source, target);
        if target_path.is_file() {
            return Ok(Some(target_path));
        }
        let fallback_path = self.find_fallback_torrent(source, target);
        let Some(fallback_path) = fallback_path else {
            return Ok(None);
        };
        let announce_url = self.shared_options.announce_url.clone();
        let indexer = self.shared_options.get_indexer();
        TorrentCreator::duplicate(&fallback_path, &target_path, announce_url, indexer).await?;
        Ok(Some(target_path))
    }

    /// Find a torrent file from another tracker that can be duplicated.
    fn find_fallback_torrent(&self, source: &Source, target: TargetFormat) -> Option<PathBuf> {
        let current_indexer = self.shared_options.get_indexer();
        for indexer in KNOWN_INDEXERS {
            if indexer == current_indexer {
                continue;
            }
            let path = self.get_torrent_path_for_indexer(source, target, indexer);
            if path.is_file() {
                return Some(path);
            }
        }
        None
    }
}
