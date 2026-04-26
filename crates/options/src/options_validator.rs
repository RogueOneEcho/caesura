//! Validator helper for collecting [`OptionIssue`]s.

use crate::OptionIssue;
use colored::Colorize;
use std::path::Path;

/// Collects [`OptionIssue`]s emitted while validating resolved options.
///
/// Use [`Self::push`] for bespoke cases not covered by a helper.
#[derive(Debug, Default)]
pub struct OptionsValidator {
    issues: Vec<OptionIssue>,
}

impl OptionsValidator {
    /// Create an empty [`Self`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Consume self and return the collected issues.
    #[must_use]
    pub fn into_issues(self) -> Vec<OptionIssue> {
        self.issues
    }

    /// Check for issues, logging any that were collected.
    ///
    /// - Returns `true` if no issues were collected.
    /// - Returns `false` if any issues were collected.
    #[must_use]
    pub fn check(&self) -> bool {
        if self.issues.is_empty() {
            return true;
        }
        log::error!("{} configuration", "Invalid".bold());
        for issue in &self.issues {
            log::warn!("{issue}");
        }
        false
    }

    /// Push a fully-constructed issue.
    pub fn push(&mut self, issue: OptionIssue) {
        self.issues.push(issue);
    }

    /// Push `Required` if `value` is `None`.
    pub fn check_set<T>(&mut self, id: &str, value: &Option<T>) {
        if value.is_none() {
            self.issues.push(OptionIssue::required(id));
        }
    }

    /// Push `UrlInvalid` if the URL does not start with `http://` or `https://`,
    /// or ends with `/`. A single issue carries one or both reasons.
    pub fn check_url(&mut self, id: &str, url: &str) {
        let mut reasons: Vec<&str> = Vec::new();
        if !url.starts_with("http://") && !url.starts_with("https://") {
            reasons.push("must start with http:// or https://");
        }
        if url.ends_with('/') {
            reasons.push("must not end with a trailing slash");
        }
        if !reasons.is_empty() {
            self.issues
                .push(OptionIssue::url_invalid(id, url, &reasons));
        }
    }

    /// Push `DirectoryNotFound` if the path is not a directory.
    pub fn check_dir_exists(&mut self, id: &str, path: &Path) {
        if !path.is_dir() {
            self.issues.push(OptionIssue::directory_not_found(id, path));
        }
    }

    /// Push `RequiredNonEmpty` if the slice is empty.
    pub fn check_non_empty<T>(&mut self, id: &str, value: &[T]) {
        if value.is_empty() {
            self.issues.push(OptionIssue::required_non_empty(id));
        }
    }

    /// Push `DependencyMissing` if `present` is true and `requires_present` is false.
    pub fn check_dependent(
        &mut self,
        id: &str,
        present: bool,
        requires_id: &str,
        requires_present: bool,
    ) {
        if present && !requires_present {
            self.issues
                .push(OptionIssue::dependency_missing(id, requires_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OptionIssueKind;

    fn run_with_validator(f: impl FnOnce(&mut OptionsValidator)) -> Vec<OptionIssue> {
        let mut validator = OptionsValidator::new();
        f(&mut validator);
        validator.into_issues()
    }

    #[test]
    fn options_validator_check_url_scheme() {
        let issues = run_with_validator(|v| v.check_url("indexer_url", "ftp://example.com"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::UrlInvalid);
        assert_eq!(issues[0].keys, vec!["indexer_url".to_owned()]);
        let reasons: Vec<&str> = issues[0]
            .additional
            .iter()
            .filter_map(|(k, v)| (k == "reason").then_some(v.as_str()))
            .collect();
        assert_eq!(reasons, vec!["must start with http:// or https://"]);
    }

    #[test]
    fn options_validator_check_url_trailing_slash() {
        let issues = run_with_validator(|v| v.check_url("indexer_url", "https://example.com/"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::UrlInvalid);
        let reasons: Vec<&str> = issues[0]
            .additional
            .iter()
            .filter_map(|(k, v)| (k == "reason").then_some(v.as_str()))
            .collect();
        assert_eq!(reasons, vec!["must not end with a trailing slash"]);
    }

    #[test]
    fn options_validator_check_url_both_problems() {
        let issues = run_with_validator(|v| v.check_url("indexer_url", "ftp://example.com/"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::UrlInvalid);
        let reasons: Vec<&str> = issues[0]
            .additional
            .iter()
            .filter_map(|(k, v)| (k == "reason").then_some(v.as_str()))
            .collect();
        assert_eq!(
            reasons,
            vec![
                "must start with http:// or https://",
                "must not end with a trailing slash"
            ]
        );
    }

    #[test]
    fn options_validator_check_url_clean() {
        let issues = run_with_validator(|v| v.check_url("indexer_url", "https://example.com"));
        assert!(issues.is_empty());
    }

    #[test]
    fn options_validator_check_dir_exists_missing() {
        let issues = run_with_validator(|v| {
            v.check_dir_exists("output", Path::new("/no/such/path/at/all"));
        });
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::DirectoryNotFound);
    }

    #[test]
    fn options_validator_check_dir_exists_present() {
        let issues = run_with_validator(|v| v.check_dir_exists("output", Path::new(".")));
        assert!(issues.is_empty());
    }

    #[test]
    fn options_validator_check_non_empty_empty() {
        let empty: Vec<u8> = Vec::new();
        let issues = run_with_validator(|v| v.check_non_empty("content", &empty));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::RequiredNonEmpty);
    }

    #[test]
    fn options_validator_check_non_empty_populated() {
        let populated = vec![1u8];
        let issues = run_with_validator(|v| v.check_non_empty("content", &populated));
        assert!(issues.is_empty());
    }

    #[test]
    fn options_validator_check_dependent_missing() {
        let issues = run_with_validator(|v| {
            v.check_dependent("upload", true, "transcode", false);
        });
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, OptionIssueKind::DependencyMissing);
        assert_eq!(
            issues[0].keys,
            vec!["upload".to_owned(), "transcode".to_owned()]
        );
    }

    #[test]
    fn options_validator_check_dependent_satisfied() {
        let issues = run_with_validator(|v| {
            v.check_dependent("upload", true, "transcode", true);
        });
        assert!(issues.is_empty());
    }
}
