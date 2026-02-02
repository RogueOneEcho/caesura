use crate::utils::TempDirectory;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::thread::current as current_thread;

/// Test directory with `output` and `cache` subdirectories.
///
/// - Wraps [`TempDirectory`] for automatic cleanup on drop
/// - Provides standard subdirectory paths for test output
pub struct TestDirectory {
    temp_dir: TempDirectory,
}

impl TestDirectory {
    /// Create a new test directory named after the current test.
    #[must_use]
    pub fn new() -> Self {
        let test_name = current_thread()
            .name()
            .expect("should be able to get test name")
            .replace("::", "_");
        Self {
            temp_dir: TempDirectory::create(&test_name),
        }
    }

    /// Path to the output subdirectory.
    #[must_use]
    pub fn output(&self) -> PathBuf {
        self.temp_dir.join("output")
    }

    /// Path to the cache subdirectory.
    #[must_use]
    pub fn cache(&self) -> PathBuf {
        self.temp_dir.join("cache")
    }

    /// Disable cleanup on drop, useful for debugging test failures.
    #[must_use]
    pub fn keep(mut self) -> Self {
        self.temp_dir = self.temp_dir.keep();
        self
    }
}

impl Deref for TestDirectory {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.temp_dir
    }
}

impl AsRef<Path> for TestDirectory {
    fn as_ref(&self) -> &Path {
        &self.temp_dir
    }
}
