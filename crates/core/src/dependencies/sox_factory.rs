//! Factory for building sox commands with the correct binary and flags.

use crate::prelude::*;

/// Factory for creating sox [`CommandInfo`] with the correct binary and flags.
///
/// Uses the configured `sox_path` and `sox_ng` options to select the binary and base arguments.
#[injectable]
pub(crate) struct SoxFactory {
    options: Ref<SoxOptions>,
}

impl SoxFactory {
    /// Create a new [`SoxFactory`] from sox options.
    #[cfg(test)]
    pub(crate) fn new(options: Ref<SoxOptions>) -> Self {
        Self { options }
    }

    /// Create a [`CommandInfo`] with the correct sox binary and base flags.
    #[must_use]
    pub(crate) fn create(&self) -> CommandInfo {
        CommandInfo {
            program: self.binary().to_owned(),
            args: self.base_args(),
        }
    }

    /// Return the sox binary name for display and invocation.
    #[must_use]
    pub(crate) fn binary(&self) -> &str {
        if let Some(path) = &self.options.sox_path {
            path.to_str().expect("sox_path should be valid UTF-8")
        } else if self.options.sox_ng {
            SOX_NG
        } else {
            SOX
        }
    }

    fn base_args(&self) -> Vec<String> {
        if self.options.sox_ng {
            vec!["--single-threaded".to_owned()]
        } else {
            Vec::new()
        }
    }
}
