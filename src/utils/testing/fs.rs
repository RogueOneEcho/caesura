use crate::built_info::PKG_NAME;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::thread::current as current_thread;
use std::time::SystemTime;

/// Directory containing sample torrent files for testing.
pub const TORRENTS_SAMPLES_DIR: &str = "samples/torrents";

/// Directory containing sample content for testing.
pub const SAMPLES_CONTENT_DIR: &str = "samples/content";

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

    /// Create a unique temporary directory using the current test name.
    ///
    /// - Automatically determines the test name from the current thread
    /// - Uses millisecond timestamp for uniqueness
    #[must_use]
    pub fn for_current_test() -> PathBuf {
        let test_name = current_thread()
            .name()
            .expect("should be able to get test name")
            .replace("::", "_");
        Self::create(&test_name)
    }
}
