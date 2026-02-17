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
fn set_rerun_triggers() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=.git/packed-refs");
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
