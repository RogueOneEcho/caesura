use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Successful result of a verify operation.
pub(crate) struct VerifySuccess {
    /// Validation issues found during verification.
    pub issues: Vec<SourceIssue>,
}

impl VerifySuccess {
    /// Whether the source passed verification with no issues.
    pub fn verified(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Serializable status of a [`VerifyCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct VerifyStatus {
    /// Whether the source passed verification.
    pub verified: bool,
    /// Validation issues found, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<SourceIssue>>,
    /// When the operation completed.
    pub completed: TimeStamp,
}

impl VerifyStatus {
    /// Create a new [`VerifyStatus`] from a command result.
    pub fn new(result: Result<VerifySuccess, Failure<VerifyAction>>) -> Self {
        match result {
            Ok(success) => {
                if success.issues.is_empty() {
                    Self {
                        verified: true,
                        issues: None,
                        completed: TimeStamp::now(),
                    }
                } else {
                    Self {
                        verified: false,
                        issues: Some(success.issues),
                        completed: TimeStamp::now(),
                    }
                }
            }
            Err(failure) => Self {
                verified: false,
                issues: Some(vec![SourceIssue::Error {
                    domain: "verify".to_owned(),
                    details: failure.to_string(),
                }]),
                completed: TimeStamp::now(),
            },
        }
    }

    #[cfg(test)]
    pub(crate) fn verified() -> Self {
        Self {
            verified: true,
            issues: None,
            completed: TimeStamp::now(),
        }
    }

    /// Create a [`VerifyStatus`] from a single [`SourceIssue`].
    pub(crate) fn from_issue(issue: SourceIssue) -> Self {
        Self {
            verified: false,
            issues: Some(vec![issue]),
            completed: TimeStamp::now(),
        }
    }
}
