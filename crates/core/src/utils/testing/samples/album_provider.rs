use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::{Mutex, OnceCell};

use super::{AlbumConfig, AlbumGenerator, SampleFormat};
use crate::utils::DiagnosticExt;

/// Per-format cache of album generation results.
///
/// - `LazyLock`: initialized on first access
/// - `Mutex`: async-safe access to the map
/// - `HashMap<SampleFormat, ...>`: one entry per format (e.g., `FLAC16_441`, `FLAC24_96`)
/// - `OnceCell`: ensures generation runs only once per format
/// - `Result<MockGazelleClient, String>`: caches success or error message
#[allow(clippy::type_complexity)]
static ALBUM_CACHE: LazyLock<Mutex<HashMap<SampleFormat, OnceCell<Result<AlbumConfig, String>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached provider for generated test albums.
pub struct AlbumProvider;

impl AlbumProvider {
    /// Generate sample files and return the album configuration.
    ///
    /// - Generates FLAC files, cover image, and torrent if not already cached
    /// - Uses file-based locking for cross-process coordination
    /// - Panics with a descriptive message if generation fails
    #[allow(clippy::panic)]
    pub async fn get(format: SampleFormat) -> AlbumConfig {
        let cell = {
            let mut map = ALBUM_CACHE.lock().await;
            map.entry(format).or_insert_with(OnceCell::new).clone()
        };
        cell.get_or_init(|| async {
            let config = AlbumConfig::with_format(format);
            AlbumGenerator::generate(&config)
                .await
                .map_err(|e| e.render())?;
            Ok(config)
        })
        .await
        .clone()
        .unwrap_or_else(|e| panic!("Sample generation failed\n{e}"))
    }
}
