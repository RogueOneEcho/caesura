use crate::built_info::PKG_NAME;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

/// Workspace root directory (two levels up from crates/core).
static WORKSPACE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Should be able to find workspace root")
        .to_path_buf()
});

/// Directory containing sample sources for testing.
pub static SAMPLE_SOURCES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| WORKSPACE_DIR.join("samples/sources"));

/// Directory containing cached transcodes for testing.
pub static SAMPLE_TRANSCODES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| WORKSPACE_DIR.join("samples/transcodes"));

/// Utility for creating timestamped temporary directories.
pub struct TempDirectory;

impl TempDirectory {
    /// Return a unique temporary directory path without creating it.
    ///
    /// - Uses millisecond timestamp for uniqueness
    #[must_use]
    pub fn get(test_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration should be valid")
            .as_millis()
            .to_string();
        temp_dir().join(PKG_NAME).join(test_name).join(timestamp)
    }

    /// Create a unique temporary directory and return its path.
    ///
    /// - Calls [`Self::get`] and creates the directory
    #[must_use]
    pub fn create(test_name: &str) -> PathBuf {
        let dir = Self::get(test_name);
        create_dir_all(&dir).expect("Should be able to create temp dir");
        dir
    }
}
