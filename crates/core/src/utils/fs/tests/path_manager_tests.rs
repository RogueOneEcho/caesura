//! Tests for [`PathManager`] default path functions.

use crate::testing_prelude::*;

/// Verify default config path ends with the expected platform suffix.
#[test]
fn path_manager_default_config_path() {
    let path = PathManager::default_config_path();
    let expected = if is_docker() {
        "/config.yml"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".config/caesura/config.yml"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Application Support/caesura/config.yml"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Roaming/caesura/config.yml"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default cache directory ends with the expected platform suffix.
#[test]
fn path_manager_default_cache_dir() {
    let path = PathManager::default_cache_dir();
    let expected = if is_docker() {
        "/cache"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".cache/caesura"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Caches/caesura"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Local/caesura"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default output directory ends with the expected platform suffix.
#[test]
fn path_manager_default_output_dir() {
    let path = PathManager::default_output_dir();
    let expected = if is_docker() {
        "/output"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".local/share/caesura/output"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Application Support/caesura/output"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Roaming/caesura/output"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}
