use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Legacy cache path from before platform user directories.
const LEGACY_CACHE_DIR: &str = "./cache";

/// Options for queue cache
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct CacheOptions {
    /// Path to cache directory.
    #[arg(long)]
    #[options(default_fn = default_cache, default_doc = "`~/.cache/caesura/` or platform equivalent")]
    pub cache: PathBuf,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_cache(_partial: &CacheOptionsPartial) -> Option<PathBuf> {
    Some(PathManager::default_cache_dir())
}

impl OptionsContract for CacheOptions {
    type Partial = CacheOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if !self.cache.is_dir() {
            errors.push(DoesNotExist(
                "Cache Directory".to_owned(),
                self.cache.to_string_lossy().to_string(),
            ));
            if PathBuf::from(LEGACY_CACHE_DIR).is_dir() {
                let default_dir = PathManager::default_cache_dir();
                errors.push(Changed(
                    "Cache Directory".to_owned(),
                    self.cache.to_string_lossy().to_string(),
                    format!("In v0.27.0 the default cache path changed to {}.\nPass the option: --cache {LEGACY_CACHE_DIR} to use the previous cache path.", default_dir.display()),
                ));
            }
        }
    }
}
