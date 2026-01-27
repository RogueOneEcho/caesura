use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use gazelle_api::MockGazelleClient;
use tokio::sync::{Mutex, OnceCell};

use super::{SampleDataBuilder, SampleError, SampleFormat};

const TIMEOUT_SECONDS: u64 = 10;
const POLL_MILLISECONDS: u64 = 500;

/// Per-format cache of sample generation results.
///
/// - `LazyLock`: initialized on first access
/// - `Mutex`: async-safe access to the map
/// - `HashMap<SampleFormat, ...>`: one entry per format (e.g., `FLAC16_441`, `FLAC24_96`)
/// - `OnceCell`: ensures generation runs only once per format
/// - `Result<MockGazelleClient, String>`: caches success or error message
#[allow(clippy::type_complexity)]
static SAMPLE_CACHE: LazyLock<
    Mutex<HashMap<SampleFormat, OnceCell<Result<MockGazelleClient, String>>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Create sample files and return a configured [`MockGazelleClient`].
///
/// - Generates FLAC files, cover image, and torrent if not already cached
/// - Uses file-based locking for cross-process coordination
/// - Panics with a descriptive message if generation fails
#[allow(clippy::panic)]
pub async fn get_samples(format: SampleFormat) -> MockGazelleClient {
    let cell = {
        let mut map = SAMPLE_CACHE.lock().await;
        map.entry(format).or_insert_with(OnceCell::new).clone()
    };
    cell.get_or_init(|| async {
        let data = SampleDataBuilder::new(format);
        create_samples_if_missing(&data)
            .await
            .and_then(|()| data.mock_client())
            .map_err(|e| e.to_string())
    })
    .await
    .clone()
    .unwrap_or_else(|e| panic!("Sample generation failed\n{e}"))
}

async fn create_samples_if_missing(data: &SampleDataBuilder) -> Result<(), SampleError> {
    let marker = PathBuf::from(data.samples_dir()).join(".generated");
    let lock = PathBuf::from(data.samples_dir()).join(".lock");
    // Fast path: already generated
    if marker.exists() {
        return Ok(());
    }
    // Try to acquire lock
    if let Some(parent) = lock.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .is_ok()
    {
        // We got the lock - generate samples
        let result = data.build().await;
        // Create marker on success
        if result.is_ok() {
            let _ = File::create(&marker);
        }
        // Release lock
        let _ = fs::remove_file(&lock);
        result
    } else {
        // Lock exists - wait for completion
        wait_for_generation(&marker, &lock);
        Ok(())
    }
}

fn wait_for_generation(marker: &Path, lock: &Path) {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let poll = Duration::from_millis(POLL_MILLISECONDS);
    let start = Instant::now();
    while start.elapsed() < timeout {
        if marker.exists() {
            return; // Generation complete
        }
        // Lock released but no marker - generation failed
        assert!(lock.exists(), "Sample generation failed in another process");
        thread::sleep(poll);
    }
    unreachable!("Timeout waiting for sample generation");
}
