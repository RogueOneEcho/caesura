//! Configuration for cached transcodes in tests.

use crate::utils::{AlbumConfig, SAMPLE_TRANSCODES_DIR, TargetFormat};
use std::path::{Path, PathBuf};

/// Configuration for a cached transcode output.
#[derive(Debug, Clone)]
pub struct TranscodeConfig {
    /// The source album configuration.
    pub album: AlbumConfig,
    /// The target format for the transcode.
    pub target: TargetFormat,
}

impl TranscodeConfig {
    /// Create a new transcode configuration.
    #[must_use]
    pub fn new(album: AlbumConfig, target: TargetFormat) -> Self {
        Self { album, target }
    }

    /// Directory name for the transcode output.
    ///
    /// Format: `Artist - Album [Year] [WEB FORMAT]`
    ///
    /// Note: This matches the naming convention used by [`TranscodeName`].
    #[must_use]
    pub fn dir_name(&self) -> String {
        format!(
            "{} - {} [{}] [WEB {}]",
            self.album.artist,
            self.album.album,
            self.album.year,
            self.target.get_name()
        )
    }

    /// Full path to the cached transcode directory.
    #[must_use]
    pub fn transcode_dir(&self) -> PathBuf {
        Path::new(SAMPLE_TRANSCODES_DIR).join(self.dir_name())
    }

    /// Full path to the torrent file.
    #[must_use]
    pub fn torrent_path(&self) -> PathBuf {
        Path::new(SAMPLE_TRANSCODES_DIR).join(format!("{}.torrent", self.dir_name()))
    }
}
