//! Configuration issue type.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;
use thiserror::Error;

/// A configuration issue identified during validation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "testing", derive(serde::Serialize))]
pub struct OptionIssue {
    /// `snake_case` field ids the issue applies to.
    /// Usually one. Multiple for `RequiredOneOf` and `DependencyMissing`. Empty for `ConfigInvalid`.
    pub keys: Vec<String>,
    /// Discriminant identifying the kind of issue.
    pub kind: OptionIssueKind,
    /// Free-form key/value context, mirroring `Failure::additional`.
    pub additional: Vec<(String, String)>,
}

/// What kind of issue this is.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[cfg_attr(feature = "testing", derive(serde::Serialize))]
pub enum OptionIssueKind {
    /// Required option is not set.
    #[error("Required")]
    Required,
    /// Required option was set but contained no values.
    #[error("Required")]
    RequiredNonEmpty,
    /// At least one option in a group must be set.
    #[error("At least one required")]
    RequiredOneOf,
    /// Option requires another option to be set.
    #[error("Dependency missing")]
    DependencyMissing,
    /// URL is malformed. One or more `reason` entries describe what is wrong.
    #[error("Invalid URL")]
    UrlInvalid,
    /// Directory does not exist.
    #[error("Directory not found")]
    DirectoryNotFound,
    /// File does not exist.
    #[error("File not found")]
    FileNotFound,
    /// Path does not exist (either file or directory accepted).
    #[error("Path not found")]
    PathNotFound,
    /// Duration string could not be parsed.
    #[error("Invalid duration")]
    DurationInvalid,
    /// Hash string could not be parsed.
    #[error("Invalid hash")]
    HashInvalid,
    /// Default for an option has changed in a recent release.
    #[error("Default changed")]
    DefaultChanged,
    /// Config file could not be read or deserialized.
    #[error("Invalid config")]
    ConfigInvalid,
    /// CLI argument extraction failed.
    #[error("Invalid CLI argument")]
    CliArgumentInvalid,
}

impl OptionIssue {
    /// Create a [`Self`] with [`OptionIssueKind::Required`] for `id`.
    #[must_use]
    pub fn required(id: &str) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::Required,
            additional: Vec::new(),
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::RequiredNonEmpty`] for `id`.
    #[must_use]
    pub fn required_non_empty(id: &str) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::RequiredNonEmpty,
            additional: Vec::new(),
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::RequiredOneOf`] for the given ids.
    #[must_use]
    pub fn required_one_of(ids: &[&str]) -> Self {
        for id in ids {
            debug_assert_snake_case(id);
        }
        Self {
            keys: ids.iter().map(|s| (*s).to_owned()).collect(),
            kind: OptionIssueKind::RequiredOneOf,
            additional: Vec::new(),
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::DependencyMissing`] indicating `id` requires `requires`.
    #[must_use]
    pub fn dependency_missing(id: &str, requires: &str) -> Self {
        debug_assert_snake_case(id);
        debug_assert_snake_case(requires);
        Self {
            keys: vec![id.to_owned(), requires.to_owned()],
            kind: OptionIssueKind::DependencyMissing,
            additional: Vec::new(),
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::UrlInvalid`] for `id`.
    ///
    /// - Each entry in `reasons` is appended as a separate `reason:` bullet.
    #[must_use]
    pub fn url_invalid(id: &str, url: &str, reasons: &[&str]) -> Self {
        debug_assert_snake_case(id);
        let mut additional = Vec::with_capacity(reasons.len() + 1);
        additional.push(("url".to_owned(), url.to_owned()));
        for reason in reasons {
            additional.push(("reason".to_owned(), (*reason).to_owned()));
        }
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::UrlInvalid,
            additional,
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::DirectoryNotFound`] for `id`.
    #[must_use]
    pub fn directory_not_found(id: &str, path: &Path) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::DirectoryNotFound,
            additional: vec![("path".to_owned(), path.display().to_string())],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::FileNotFound`] for `id`.
    #[must_use]
    pub fn file_not_found(id: &str, path: &Path) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::FileNotFound,
            additional: vec![("path".to_owned(), path.display().to_string())],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::PathNotFound`] for `id`.
    #[must_use]
    pub fn path_not_found(id: &str, path: &Path) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::PathNotFound,
            additional: vec![("path".to_owned(), path.display().to_string())],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::DurationInvalid`] for `id`.
    #[must_use]
    pub fn duration_invalid(id: &str, duration: &str, reason: &str) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::DurationInvalid,
            additional: vec![
                ("duration".to_owned(), duration.to_owned()),
                ("reason".to_owned(), reason.to_owned()),
            ],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::HashInvalid`] for `id`.
    #[must_use]
    pub fn hash_invalid(id: &str, hash: &str, reason: &str) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::HashInvalid,
            additional: vec![
                ("hash".to_owned(), hash.to_owned()),
                ("reason".to_owned(), reason.to_owned()),
            ],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::DefaultChanged`] for `id`.
    #[must_use]
    pub fn default_changed(id: &str, value: &str, details: &str) -> Self {
        debug_assert_snake_case(id);
        Self {
            keys: vec![id.to_owned()],
            kind: OptionIssueKind::DefaultChanged,
            additional: vec![
                ("value".to_owned(), value.to_owned()),
                ("details".to_owned(), details.to_owned()),
            ],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::ConfigInvalid`] from a serde error message.
    #[must_use]
    pub fn config_invalid(reason: &str) -> Self {
        Self {
            keys: Vec::new(),
            kind: OptionIssueKind::ConfigInvalid,
            additional: vec![("reason".to_owned(), reason.to_owned())],
        }
    }

    /// Create a [`Self`] with [`OptionIssueKind::CliArgumentInvalid`].
    ///
    /// - Theoretically unreachable: clap exits the process on its own parse
    ///   errors, and partial fields are all `Option<T>` so `from_arg_matches`
    ///   deserialization is not expected to fail.
    /// - `name` is the options struct name (not a `snake_case` field id) since
    ///   no field-level information is available at this call site.
    /// - Keys is left empty so no `--flag` line is rendered.
    #[must_use]
    pub fn cli_argument_invalid(name: &str, reason: &str) -> Self {
        Self {
            keys: Vec::new(),
            kind: OptionIssueKind::CliArgumentInvalid,
            additional: vec![
                ("options".to_owned(), name.to_owned()),
                ("reason".to_owned(), reason.to_owned()),
            ],
        }
    }
}

impl Display for OptionIssue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.kind)?;
        match self.kind {
            OptionIssueKind::DependencyMissing => {
                if let Some(key) = self.keys.first() {
                    write!(formatter, "\n▷ key: {key}")?;
                    write!(formatter, "\n▷ flag: --{}", snake_to_kebab(key))?;
                }
                if let Some(key) = self.keys.get(1) {
                    write!(formatter, "\n▷ requires key: {key}")?;
                    write!(formatter, "\n▷ requires flag: --{}", snake_to_kebab(key))?;
                }
            }
            _ => {
                for key in &self.keys {
                    write!(formatter, "\n▷ key: {key}")?;
                    write!(formatter, "\n▷ flag: --{}", snake_to_kebab(key))?;
                }
            }
        }
        for (key, value) in &self.additional {
            write!(formatter, "\n▷ {key}: {value}")?;
        }
        Ok(())
    }
}

fn snake_to_kebab(id: &str) -> String {
    id.replace('_', "-")
}

fn is_snake_case(id: &str) -> bool {
    let bytes = id.as_bytes();
    let Some(&first) = bytes.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    if bytes.last() == Some(&b'_') {
        return false;
    }
    let mut prev_underscore = false;
    for &b in bytes {
        let is_underscore = b == b'_';
        if is_underscore && prev_underscore {
            return false;
        }
        if !(b.is_ascii_lowercase() || b.is_ascii_digit() || is_underscore) {
            return false;
        }
        prev_underscore = is_underscore;
    }
    true
}

fn debug_assert_snake_case(id: &str) {
    debug_assert!(
        is_snake_case(id),
        "OptionIssue id must be snake_case, got: {id}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn option_issue_display_required() {
        let issue = OptionIssue::required("indexer_url");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_required_non_empty() {
        let issue = OptionIssue::required_non_empty("content");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_required_one_of() {
        let issue = OptionIssue::required_one_of(&["indexer_url", "announce_url"]);
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_dependency_missing() {
        let issue = OptionIssue::dependency_missing("upload", "transcode");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_url_invalid_scheme() {
        let issue = OptionIssue::url_invalid(
            "indexer_url",
            "not-a-valid-url.example",
            &["must start with http:// or https://"],
        );
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_url_invalid_multiple_reasons() {
        let issue = OptionIssue::url_invalid(
            "indexer_url",
            "ftp://example.com/",
            &[
                "must start with http:// or https://",
                "must not end with a trailing slash",
            ],
        );
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_directory_not_found() {
        let issue = OptionIssue::directory_not_found("output", Path::new("/no/such/dir"));
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_file_not_found() {
        let issue = OptionIssue::file_not_found("config", Path::new("/no/such/file.yml"));
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_path_not_found() {
        let issue = OptionIssue::path_not_found("queue_add_path", Path::new("/no/such/path"));
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_duration_invalid() {
        let issue = OptionIssue::duration_invalid(
            "wait_before_upload",
            "not-a-duration",
            "expected number followed by unit",
        );
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_hash_invalid() {
        let issue = OptionIssue::hash_invalid("queue_rm_hash", "zz", "invalid hex character");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_default_changed() {
        let issue =
            OptionIssue::default_changed("output", "./output", "Default changed in v0.27.0.");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_config_invalid() {
        let issue = OptionIssue::config_invalid("missing field `indexer` at line 3 column 1");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    fn option_issue_display_cli_argument_invalid() {
        let issue = OptionIssue::cli_argument_invalid("SharedOptions", "missing required argument");
        assert_snapshot!(issue.to_string());
    }

    #[test]
    #[should_panic(expected = "OptionIssue id must be snake_case")]
    fn option_issue_required_kebab_case_id() {
        let _ = OptionIssue::required("indexer-url");
    }
}
