//! Create a compile-time environment variable from [`git describe`] output.
//!
//! This includes:
//! - Latest `v*` tag (e.g. `v0.26.0`)
//! - Number of commits since tag (e.g. `v0.26.0-3-gabcdef`)
//! - Whether there are uncommitted changes (e.g. `v0.26.0-dirty`)
//! - Falls back to the commit hash when no `v*` tag is reachable (e.g. `abcdef`)
//!
//! No other git metadata is captured. Branch name, author, remote url, etc will never be exposed.
//!
//! This is used to determine if this is a release or development build.
//!
//! - <https://git-scm.com/docs/git-describe>

use std::path::PathBuf;
use std::process::Command;

fn main() {
    set_rerun_triggers();
    if let Some(describe) = git_describe() {
        set_env("CAESURA_GIT_DESCRIBE", &describe);
    }
}

/// Tell Cargo when to re-run this build script.
///
/// - `.git/HEAD` changes on branch switches.
/// - `.git/refs` (directory) changes when refs are created or deleted,
///   catching new tags and branches.
/// - `.git/packed-refs` changes when git packs loose refs or updates
///   an existing packed ref (e.g. a new commit on the current branch).
///
/// Paths are resolved from the workspace root because `rerun-if-changed`
/// is relative to the crate's `Cargo.toml` directory (`crates/core/`),
/// not the workspace root where `.git/` lives.
fn set_rerun_triggers() {
    let git_dir = workspace_root().join(".git");
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    println!("cargo:rerun-if-changed={}", git_dir.join("refs").display());
    println!(
        "cargo:rerun-if-changed={}",
        git_dir.join("packed-refs").display()
    );
}

/// Resolve the workspace root from `CARGO_MANIFEST_DIR`.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .expect("workspace root should contain Cargo.lock")
        .to_path_buf()
}

/// Run `git describe --tags --dirty --always` and return the trimmed output.
///
/// Returns `None` if git is not installed, the command fails (e.g. shallow
/// clone, zip download), or the output is empty.
fn git_describe() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--match", "v*", "--dirty", "--always"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let describe = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if describe.is_empty() {
        return None;
    }
    Some(describe)
}

/// Set a compile-time environment variable via `cargo:rustc-env`.
///
/// The variable is baked into the binary and readable with `env!()` or
/// `option_env!()` at compile time.
fn set_env(key: &str, value: &str) {
    println!("cargo:rustc-env={key}={value}");
}
