//! Platform-aware snapshot assertion macros.

/// Assert YAML snapshot exactly, or just check output is non-empty on ARM and Nix.
///
/// Transcode output hashes differ on ARM and when `sox_ng` is built from source via Nix.
macro_rules! assert_transcode_snapshot {
    ($snapshot:expr) => {
        if is_nix() || is_aarch64() {
            assert!(!$snapshot.is_empty(), "should produce transcode files");
        } else {
            assert_yaml_snapshot!($snapshot);
        }
    };
}

/// Assert string snapshot exactly, or just check output is non-empty on ARM and Nix.
///
/// Inspect output includes metadata that differs on ARM and Nix builds.
macro_rules! assert_inspect_snapshot {
    ($output:expr) => {
        if is_nix() || is_aarch64() {
            assert!(!$output.is_empty(), "should produce inspect output");
        } else {
            insta::assert_snapshot!($output);
        }
    };
}

pub(crate) use assert_inspect_snapshot;
pub(crate) use assert_transcode_snapshot;
