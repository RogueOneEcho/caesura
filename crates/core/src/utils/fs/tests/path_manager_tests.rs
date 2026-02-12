//! Tests for [`PathManager`] default path functions.

use crate::prelude::*;

/// Verify default config path ends with the expected platform suffix.
#[test]
fn default_config_path() {
    let path = PathManager::default_config_path();
    #[cfg(target_os = "linux")]
    let expected = ".config/caesura/config.yml";
    #[cfg(target_os = "macos")]
    let expected = "Library/Application Support/caesura/config.yml";
    #[cfg(target_os = "windows")]
    let expected = "AppData/Roaming/caesura/config.yml";
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default cache directory ends with the expected platform suffix.
#[test]
fn default_cache_dir() {
    let path = PathManager::default_cache_dir();
    #[cfg(target_os = "linux")]
    let expected = ".cache/caesura";
    #[cfg(target_os = "macos")]
    let expected = "Library/Caches/caesura";
    #[cfg(target_os = "windows")]
    let expected = "AppData/Local/caesura";
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default output directory ends with the expected platform suffix.
#[test]
fn default_output_dir() {
    let path = PathManager::default_output_dir();
    #[cfg(target_os = "linux")]
    let expected = ".local/share/caesura/output";
    #[cfg(target_os = "macos")]
    let expected = "Library/Application Support/caesura/output";
    #[cfg(target_os = "windows")]
    let expected = "AppData/Roaming/caesura/output";
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}
