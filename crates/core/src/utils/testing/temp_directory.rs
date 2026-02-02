//! RAII wrapper for temporary directories with automatic cleanup.

use crate::built_info::PKG_NAME;
use chrono::Local;
use std::env::temp_dir;
use std::fs::{create_dir_all, remove_dir_all};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic counter to guarantee unique paths when multiple directories are created
/// in the same second (e.g., parallel tests). Combined with ISO timestamp to form
/// the unique directory suffix: `{timestamp}-{counter}`.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Temporary directory with automatic cleanup on drop.
///
/// - Creates a timestamped directory under `/tmp/{PKG_NAME}/{test_name}/`
/// - Automatically deletes the directory when dropped (unless `keep()` is called)
/// - Implements `Deref<Target = Path>` for ergonomic path operations
pub struct TempDirectory {
    path: PathBuf,
    keep: bool,
}

impl TempDirectory {
    /// Create a new temporary directory with the given name.
    ///
    /// - Uses ISO timestamp + atomic counter for uniqueness
    /// - Creates the directory immediately
    #[must_use]
    pub fn create(test_name: &str) -> Self {
        let path = unique_path(test_name);
        create_dir_all(&path).expect("Should be able to create temp dir");
        Self { path, keep: false }
    }

    /// Disable cleanup on drop, useful for debugging test failures.
    ///
    /// - Returns self to allow chaining: `TempDirectory::create("test").keep()`
    #[must_use]
    pub fn keep(mut self) -> Self {
        self.keep = true;
        self
    }
}

impl Deref for TempDirectory {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for TempDirectory {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        if !self.keep {
            let _ = remove_dir_all(&self.path);
        }
    }
}

/// Build a unique path: `/tmp/{PKG_NAME}/{test_name}/{timestamp}-{counter}`
fn unique_path(test_name: &str) -> PathBuf {
    let timestamp = Local::now().format("%Y-%m-%dT%H_%M_%S");
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    temp_dir()
        .join(PKG_NAME)
        .join(test_name)
        .join(format!("{timestamp}-{counter}"))
}
