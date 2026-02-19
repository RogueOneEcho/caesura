use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::OnceCell;

use super::album_generator::AlbumGenerator;
use super::{AlbumConfig, SampleFormat};
use crate::utils::{DiagnosticExt, SAMPLE_SOURCES_DIR};

/// Per-config cache of album generation results.
///
/// - `LazyLock`: initialized on first access
/// - `Mutex`: thread-safe access to the map
/// - `HashMap<String, ...>`: keyed by [`AlbumConfig::dir_name`]
/// - `Arc<OnceCell>`: shared across clones so concurrent callers await the same cell
/// - `Result<AlbumConfig, String>`: caches success or error message
#[expect(
    clippy::type_complexity,
    reason = "type alias would obscure cache structure"
)]
static ALBUM_CACHE: LazyLock<Mutex<HashMap<String, Arc<OnceCell<Result<AlbumConfig, String>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached provider for generated test albums.
pub struct AlbumProvider;

impl AlbumProvider {
    /// Generate sample files for the given format and return the album configuration.
    ///
    /// Convenience wrapper around [`Self::get_advanced`] for the common single-format case.
    pub async fn get(format: SampleFormat) -> AlbumConfig {
        Self::get_advanced(AlbumConfig::with_format(format)).await
    }

    /// Generate sample files for an arbitrary album configuration.
    ///
    /// - Returns a cached result if this config's [`AlbumConfig::dir_name`] was already generated
    /// - Otherwise generates FLAC files, cover image, and torrent
    /// - Panics with a descriptive message if generation fails
    #[expect(
        clippy::panic,
        reason = "sample generation is unrecoverable. clear message is rendered"
    )]
    pub async fn get_advanced(config: AlbumConfig) -> AlbumConfig {
        let key = config.dir_name();
        let cell = {
            let mut map = ALBUM_CACHE
                .lock()
                .expect("album cache mutex should not be poisoned");
            Arc::clone(map.entry(key).or_insert_with(|| Arc::new(OnceCell::new())))
        };
        cell.get_or_init(|| async {
            let source_dir = SAMPLE_SOURCES_DIR.join(config.dir_name());
            if !is_generated(&source_dir) {
                AlbumGenerator::generate_files(&config, &source_dir)
                    .await
                    .map_err(|e| e.render())?;
                mark_generated(&source_dir)?;
            }
            Ok(config)
        })
        .await
        .clone()
        .unwrap_or_else(|e| panic!("Sample generation failed\n{e}"))
    }
}

/// Check whether the `.generated` marker exists for a directory.
fn is_generated(dir: &Path) -> bool {
    dir.join(".generated").exists()
}

/// Create the `.generated` marker indicating successful generation.
fn mark_generated(dir: &Path) -> Result<(), String> {
    let path = dir.join(".generated");
    File::create(&path).map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    Ok(())
}
