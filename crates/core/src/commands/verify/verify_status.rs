use crate::prelude::*;

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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// Create a new [`VerifyStatus`] from a [`VerifySuccess`].
    pub fn new(success: VerifySuccess) -> Self {
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

    /// Create a [`VerifyStatus`] representing a successful verification.
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
