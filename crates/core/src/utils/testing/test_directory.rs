use crate::utils::TempDirectory;
use std::path::PathBuf;
use std::thread::current as current_thread;

/// Test directory with `output` and `cache` subdirectories.
pub struct TestDirectory {
    /// Root path of the test directory.
    pub path: PathBuf,
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
            path: TempDirectory::create(&test_name),
        }
    }

    /// Path to the output subdirectory.
    #[must_use]
    pub fn output(&self) -> PathBuf {
        self.path.join("output")
    }

    /// Path to the cache subdirectory.
    #[must_use]
    pub fn cache(&self) -> PathBuf {
        self.path.join("cache")
    }
}
