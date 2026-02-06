//! RAII file-based lock for cross-process coordination during sample generation.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT_SECONDS: u64 = 5;
const POLL_MILLISECONDS: u64 = 500;

/// RAII guard that removes a `.lock` file on drop.
///
/// Ensures lock files are cleaned up even if generation fails or panics.
pub struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Outcome of attempting to acquire a file-based generation lock.
pub enum LockOutcome {
    /// Lock acquired — caller should generate, then create the marker file.
    Acquired(LockGuard),
    /// Another process already generated the output.
    AlreadyGenerated,
}

/// Attempt to acquire a generation lock for `dir`.
///
/// - Returns [`LockOutcome::AlreadyGenerated`] if the `.generated` marker exists
/// - Acquires `.lock` and returns [`LockOutcome::Acquired`] if available
/// - Waits for another holder to finish if `.lock` is already held
pub fn acquire_generation_lock(dir: &Path) -> LockOutcome {
    let marker = dir.join(".generated");
    let lock = dir.join(".lock");
    if marker.exists() {
        return LockOutcome::AlreadyGenerated;
    }
    if let Some(parent) = lock.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .is_ok()
    {
        LockOutcome::Acquired(LockGuard { path: lock })
    } else {
        wait_for_generation(&marker, &lock);
        LockOutcome::AlreadyGenerated
    }
}

/// Create the `.generated` marker indicating successful generation.
pub fn mark_generated(dir: &Path) {
    let marker = dir.join(".generated");
    let _ = File::create(&marker);
}

fn wait_for_generation(marker: &Path, lock: &Path) {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let poll = Duration::from_millis(POLL_MILLISECONDS);
    let start = Instant::now();
    while start.elapsed() < timeout {
        if marker.exists() {
            return;
        }
        assert!(lock.exists(), "Sample generation failed in another process");
        thread::sleep(poll);
    }
    unreachable!("Timeout waiting for sample generation");
}
