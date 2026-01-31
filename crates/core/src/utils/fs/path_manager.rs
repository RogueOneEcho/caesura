use crate::prelude::*;
use std::fs::create_dir;

/// Supported tracker suffixes for cross-tracker torrent duplication.
const TRACKER_SUFFIXES: &[&str] = &["red", "ops", "pth"];

#[injectable]
pub struct PathManager {
    shared_options: Ref<SharedOptions>,
    cache_options: Ref<CacheOptions>,
    file_options: Ref<FileOptions>,
}

impl PathManager {
    #[must_use]
    pub fn get_cache_dir(&self) -> PathBuf {
        self.cache_options.cache.clone()
    }

    #[must_use]
    pub fn get_source_torrent_path(&self, source: &Source) -> PathBuf {
        let id = source.torrent.id;
        let indexer = self.shared_options.indexer_lowercase();
        let torrents_dir = self.get_cache_dir().join("torrents");
        if !torrents_dir.is_dir() {
            let _ = create_dir(&torrents_dir);
        }
        torrents_dir.join(format!("{id}.{indexer}.torrent"))
    }

    #[must_use]
    pub fn get_output_dir(&self) -> PathBuf {
        self.shared_options.output.clone()
    }

    #[must_use]
    pub fn get_spectrogram_dir(&self, source: &Source) -> PathBuf {
        self.get_output_dir()
            .join(SpectrogramName::get(&source.metadata))
    }

    #[must_use]
    pub fn get_transcode_target_dir(&self, source: &Source, target: TargetFormat) -> PathBuf {
        self.get_output_dir()
            .join(TranscodeName::get(&source.metadata, target))
    }

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

    #[must_use]
    pub fn get_torrent_path(&self, source: &Source, target: TargetFormat) -> PathBuf {
        let indexer = self.shared_options.indexer_lowercase();
        self.get_torrent_path_for_indexer(source, target, &indexer)
    }

    #[must_use]
    fn get_torrent_path_for_indexer(
        &self,
        source: &Source,
        target: TargetFormat,
        indexer: &str,
    ) -> PathBuf {
        let mut filename = TranscodeName::get(&source.metadata, target);
        filename.push('.');
        filename.push_str(indexer);
        filename.push_str(".torrent");
        self.get_output_dir().join(filename)
    }

    /// Get the torrent path if it exists, or duplicate from another tracker's torrent.
    ///
    /// Example: `path/to/Artist - Album [2012] [WEB FLAC].red.torrent`
    ///
    /// Returns `None` if no torrent exists and none can be duplicated.
    ///
    /// Returns the torrent path if it already exists for the current indexer.
    ///
    /// Or attempts to duplicate from another tracker's torrent file
    /// (e.g., `.ops.torrent`, `.pth.torrent`, `.red.torrent`).
    ///
    /// Returns the torrent path if duplication is successful, else `None`.
    pub async fn get_or_duplicate_existing_torrent_path(
        &self,
        source: &Source,
        target: TargetFormat,
    ) -> Result<Option<PathBuf>, Error> {
        let target_path = self.get_torrent_path(source, target);
        if target_path.is_file() {
            return Ok(Some(target_path));
        }
        let fallback_path = self.find_fallback_torrent(source, target);
        let Some(fallback_path) = fallback_path else {
            return Ok(None);
        };
        let transcode_dir = self.get_transcode_target_dir(source, target);
        let announce_url = self
            .shared_options
            .announce_url
            .clone()
            .expect("announce should be set");
        let indexer = self.shared_options.indexer_lowercase();
        let success = ImdlCommand::duplicate_torrent(
            &fallback_path,
            &target_path,
            &transcode_dir,
            announce_url,
            indexer,
        )
        .await?;
        if success {
            Ok(Some(target_path))
        } else {
            Ok(None)
        }
    }

    /// Find a torrent file from another tracker that can be duplicated.
    fn find_fallback_torrent(&self, source: &Source, target: TargetFormat) -> Option<PathBuf> {
        let current_indexer = self.shared_options.indexer_lowercase();
        for suffix in TRACKER_SUFFIXES {
            if *suffix == current_indexer {
                continue;
            }
            let path = self.get_torrent_path_for_indexer(source, target, suffix);
            if path.is_file() {
                return Some(path);
            }
        }
        None
    }
}
