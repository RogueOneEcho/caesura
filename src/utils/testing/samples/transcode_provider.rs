//! Cached provider for transcode outputs in tests.

use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::{Mutex, OnceCell};

use super::{AlbumProvider, SampleFormat, TranscodeConfig, TranscodeGenerator};
use crate::utils::{AlbumConfig, TargetFormat};

/// Cache key combining source format and target format.
type TranscodeCacheKey = (SampleFormat, TargetFormat);

/// Per-format cache of transcode generation results.
///
/// - `LazyLock`: initialized on first access
/// - `Mutex`: async-safe access to the map
/// - `HashMap<(SampleFormat, TargetFormat), ...>`: one entry per source/target combination
/// - `OnceCell`: ensures generation runs only once per combination
/// - `Result<TranscodeConfig, String>`: caches success or error message
#[allow(clippy::type_complexity)]
static TRANSCODE_CACHE: LazyLock<
    Mutex<HashMap<TranscodeCacheKey, OnceCell<Result<TranscodeConfig, String>>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached provider for generated transcode outputs.
pub struct TranscodeProvider;

impl TranscodeProvider {
    /// Generate transcode files and return the transcode configuration.
    ///
    /// - First ensures the source album exists via [`AlbumProvider::get`]
    /// - Then generates the transcode if not already cached
    /// - Uses file-based locking for cross-process coordination
    /// - Panics with a descriptive message if generation fails
    #[allow(clippy::panic)]
    pub async fn get(source: SampleFormat, target: TargetFormat) -> TranscodeConfig {
        // First ensure the source album is generated
        let album = AlbumProvider::get(source).await;

        // Then get or generate the transcode
        let key = (source, target);
        let cell = {
            let mut map = TRANSCODE_CACHE.lock().await;
            map.entry(key).or_insert_with(OnceCell::new).clone()
        };
        cell.get_or_init(|| async {
            let config = TranscodeConfig::new(album.clone(), target);
            TranscodeGenerator::generate(&config)
                .await
                .map_err(|e| e.to_string())?;
            Ok(config)
        })
        .await
        .clone()
        .unwrap_or_else(|e| panic!("Transcode generation failed\n{e}"))
    }

    /// Get a transcode configuration for a custom album config.
    ///
    /// This variant allows using custom album configurations (e.g., multi-disc albums)
    /// rather than the default test album.
    ///
    /// Note: This does not use caching, so each call will check/generate the transcode.
    #[allow(dead_code)]
    #[allow(clippy::panic)]
    pub async fn get_for_album(album: &AlbumConfig, target: TargetFormat) -> TranscodeConfig {
        let config = TranscodeConfig::new(album.clone(), target);
        TranscodeGenerator::generate(&config)
            .await
            .unwrap_or_else(|e| panic!("Transcode generation failed\n{e}"));
        config
    }
}
