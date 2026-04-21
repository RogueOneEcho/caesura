//! Cached provider for transcode outputs in tests.

use super::transcode_generator::TranscodeGenerator;
use crate::testing_prelude::*;
use std::sync::Mutex;

/// Per-config cache of transcode generation results.
///
/// - `LazyLock`: initialized on first access
/// - `Mutex`: thread-safe access to the map
/// - `HashMap<String, ...>`: keyed by [`TranscodeConfig::dir_name`]
/// - `Arc<OnceCell>`: shared across clones so concurrent callers await the same cell
/// - `Result<TranscodeConfig, String>`: caches success or error message
#[expect(
    clippy::type_complexity,
    reason = "type alias would obscure cache structure"
)]
static TRANSCODE_CACHE: LazyLock<
    Mutex<HashMap<String, Arc<OnceCell<Result<TranscodeConfig, String>>>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached provider for generated transcode outputs.
pub struct TranscodeProvider;

impl TranscodeProvider {
    /// Generate transcode files and return the transcode configuration.
    ///
    /// - First ensures the source album exists via [`AlbumProvider::get`]
    /// - Then generates the transcode if not already cached
    /// - Panics with a descriptive message if generation fails
    pub async fn get(source: SampleFormat, target: TargetFormat) -> TranscodeConfig {
        let album = AlbumProvider::get(source).await;
        Self::get_advanced(&album, target).await
    }

    /// Generate a transcode for a custom album configuration.
    ///
    /// - First ensures the source album exists via [`AlbumProvider::get_advanced`]
    /// - Uses the same cache as [`Self::get`], keyed by [`TranscodeConfig::dir_name`]
    /// - Panics with a descriptive message if generation fails
    #[expect(
        clippy::panic,
        reason = "sample generation is unrecoverable. clear message is rendered"
    )]
    pub async fn get_advanced(album: &AlbumConfig, target: TargetFormat) -> TranscodeConfig {
        let config = TranscodeConfig::new(album.clone(), target);
        let key = config.dir_name();
        let cell = {
            let mut map = TRANSCODE_CACHE
                .lock()
                .expect("transcode cache mutex should not be poisoned");
            Arc::clone(map.entry(key).or_insert_with(|| Arc::new(OnceCell::new())))
        };
        cell.get_or_init(|| async {
            let transcode_dir = config.transcode_dir();
            if !is_generated(&transcode_dir) {
                TranscodeGenerator::generate_files(&config, &transcode_dir)
                    .await
                    .map_err(|e| e.render())?;
                mark_generated(&transcode_dir)?;
            }
            Ok(config)
        })
        .await
        .clone()
        .unwrap_or_else(|e| panic!("Transcode generation failed\n{e}"))
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
