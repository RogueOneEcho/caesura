//! Application-level metadata: name, version, build status, and user agent.

use std::fmt;

/// Application name.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
/// Application version.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Application homepage URL.
pub const APP_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// Git describe output captured at build time, if available.
const GIT_DESCRIBE: Option<&str> = option_env!("CAESURA_GIT_DESCRIBE");

/// Version string used for development/source builds.
const DEV_VERSION: &str = "0.0.0";

/// Whether the binary is an official release, an unmodified source build,
/// a modified source build, or of unknown provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildStatus {
    /// Official release binary with a version other than `0.0.0`.
    Release,
    /// Source build from a clean, tagged checkout.
    Unmodified,
    /// Source build with local modifications or commits ahead of a tag.
    Modified,
    /// Source build where git information is unavailable.
    Unknown,
}

impl fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl BuildStatus {
    /// Label as a string slice.
    fn as_str(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Unmodified => "unmodified",
            Self::Modified => "modified",
            Self::Unknown => "unknown",
        }
    }
    /// Determine the build status of this binary.
    pub fn get() -> Self {
        Self::from_version_and_describe(APP_VERSION, GIT_DESCRIBE)
    }

    /// Determine the build status from a version string and git describe output.
    fn from_version_and_describe(version: &str, describe: Option<&str>) -> Self {
        if version != DEV_VERSION {
            return Self::Release;
        }
        let Some(desc) = describe else {
            return Self::Unknown;
        };
        if desc.contains("-dirty") {
            return Self::Modified;
        }
        // Pattern: v1.2.3-N-gabcdef means N commits ahead of tag
        if desc.contains("-g") {
            return Self::Modified;
        }
        // Clean tag match (e.g. "v0.26.0")
        if desc.starts_with('v') {
            return Self::Unmodified;
        }
        Self::Unknown
    }
}

/// Human-readable version string.
///
/// - Release build: the Cargo package version (e.g. `v0.26.0`)
/// - Dev build with git: the git describe output (e.g. `v0.26.0-3-gabcdef`)
/// - Dev build without git: `unknown`
pub fn app_version_or_describe() -> String {
    if APP_VERSION != DEV_VERSION {
        return format!("v{APP_VERSION}");
    }
    GIT_DESCRIBE.unwrap_or(DEV_VERSION).to_owned()
}

/// Format a User-Agent string.
///
/// When `include_url` is true, appends the homepage URL (for HTTP requests).
/// When false, omits it (for display to the user).
///
/// - Release: `caesura/0.26.0 (release; https://...)`
/// - Unmodified: `caesura/0.0.0 (unmodified; v0.26.0; https://...)`
/// - Modified: `caesura/0.0.0 (modified; v0.26.0-3-gabcdef-dirty; https://...)`
/// - Unknown with git: `caesura/0.0.0 (unknown; abcdef; https://...)`
/// - Unknown without git: `caesura/0.0.0 (unknown; unknown; https://...)`
pub fn app_user_agent(include_url: bool) -> String {
    let status = BuildStatus::get();
    let mut parts: Vec<&str> = vec![status.as_str()];
    if status != BuildStatus::Release {
        parts.push(GIT_DESCRIBE.unwrap_or("unknown"));
    }
    if include_url {
        parts.push(APP_HOMEPAGE);
    }
    let comment = parts.join("; ");
    format!("{APP_NAME}/{APP_VERSION} ({comment})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_release() {
        assert_eq!(BuildStatus::Release.to_string(), "release");
    }

    #[test]
    fn display_unmodified() {
        assert_eq!(BuildStatus::Unmodified.to_string(), "unmodified");
    }

    #[test]
    fn display_modified() {
        assert_eq!(BuildStatus::Modified.to_string(), "modified");
    }

    #[test]
    fn display_unknown() {
        assert_eq!(BuildStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn build_status_release_version() {
        assert_eq!(
            BuildStatus::from_version_and_describe("1.2.3", None),
            BuildStatus::Release
        );
    }

    #[test]
    fn build_status_release_ignores_describe() {
        assert_eq!(
            BuildStatus::from_version_and_describe("1.2.3", Some("v1.2.3-dirty")),
            BuildStatus::Release,
        );
    }

    #[test]
    fn build_status_dev_no_git() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", None),
            BuildStatus::Unknown
        );
    }

    #[test]
    fn build_status_dev_clean_tag() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", Some("v0.26.0")),
            BuildStatus::Unmodified,
        );
    }

    #[test]
    fn build_status_dev_dirty_tag() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", Some("v0.26.0-dirty")),
            BuildStatus::Modified,
        );
    }

    #[test]
    fn build_status_dev_commits_ahead() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", Some("v0.26.0-3-gabcdef")),
            BuildStatus::Modified,
        );
    }

    #[test]
    fn build_status_dev_commits_ahead_dirty() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", Some("v0.26.0-3-gabcdef-dirty")),
            BuildStatus::Modified,
        );
    }

    #[test]
    fn build_status_dev_hash_only() {
        assert_eq!(
            BuildStatus::from_version_and_describe("0.0.0", Some("abcdef")),
            BuildStatus::Unknown,
        );
    }

    #[test]
    fn build_status_is_not_release() {
        let status = BuildStatus::get();
        assert_ne!(status, BuildStatus::Release);
    }

    #[test]
    fn app_user_agent_contains_status() {
        let ua = app_user_agent(true);
        let status = BuildStatus::get();
        assert!(ua.contains(&status.to_string()));
    }

    #[test]
    fn app_user_agent_contains_name_and_version() {
        let ua = app_user_agent(true);
        assert!(ua.contains(APP_NAME));
        assert!(ua.contains(APP_VERSION));
    }

    #[test]
    fn app_name_is_populated() {
        assert!(!APP_NAME.is_empty());
    }

    #[test]
    fn app_version_is_populated() {
        assert!(!APP_VERSION.is_empty());
    }

    #[test]
    fn app_homepage_is_populated() {
        assert!(!APP_HOMEPAGE.is_empty());
    }
}
