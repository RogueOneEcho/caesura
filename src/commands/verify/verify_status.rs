use crate::commands::*;
use crate::utils::*;

use serde::{Deserialize, Serialize};

/// Result of a [`VerifyCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct VerifyStatus {
    /// Whether the source passed verification.
    pub verified: bool,
    /// Issues found during verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<SourceIssue>>,
    /// Time the verification completed.
    pub completed: TimeStamp,
}

impl VerifyStatus {
    pub(crate) fn verified() -> Self {
        Self {
            verified: true,
            issues: None,
            completed: TimeStamp::now(),
        }
    }
    pub(crate) fn from_issues(issues: Vec<SourceIssue>) -> Self {
        if issues.is_empty() {
            Self::verified()
        } else {
            Self {
                verified: false,
                issues: Some(issues),
                completed: TimeStamp::now(),
            }
        }
    }
    pub(crate) fn from_issue(issue: SourceIssue) -> Self {
        Self {
            verified: false,
            issues: Some(vec![issue]),
            completed: TimeStamp::now(),
        }
    }
}
