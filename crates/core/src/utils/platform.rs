//! Platform and environment detection.

use std::env;

/// Environment variable set in the Docker image to use container paths.
const DOCKER_ENV_VAR: &str = "CAESURA_DOCKER";

/// Environment variable set in the Nix derivation to relax snapshot assertions.
#[cfg(test)]
const NIX_ENV_VAR: &str = "CAESURA_NIX";

/// Check if running in a Docker container.
pub(crate) fn is_docker() -> bool {
    env::var(DOCKER_ENV_VAR).is_ok()
}

/// Check if running in a Nix build.
#[cfg(test)]
pub(crate) fn is_nix() -> bool {
    env::var(NIX_ENV_VAR).is_ok()
}

/// Check if running on macOS.
#[cfg(test)]
pub(crate) fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Check if running on ARM (aarch64).
#[cfg(test)]
pub(crate) fn is_aarch64() -> bool {
    cfg!(target_arch = "aarch64")
}
