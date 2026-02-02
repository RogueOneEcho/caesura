use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Workspace root directory (two levels up from crates/core).
static WORKSPACE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Should be able to find workspace root")
        .to_path_buf()
});

/// Directory containing sample sources for testing.
pub static SAMPLE_SOURCES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| WORKSPACE_DIR.join("samples/sources"));

/// Directory containing cached transcodes for testing.
pub static SAMPLE_TRANSCODES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| WORKSPACE_DIR.join("samples/transcodes"));
