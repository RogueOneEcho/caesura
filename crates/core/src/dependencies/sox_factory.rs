//! Factory for building sox commands with the correct binary and flags.

use crate::prelude::*;

/// Factory for creating sox [`CommandInfo`] with the correct binary and flags.
///
/// When `--no-sox-ng` is set, uses the original `SoX` binary and omits the
/// `--single-threaded` flag.
#[injectable]
pub(crate) struct SoxFactory {
    options: Ref<SoxOptions>,
}

impl SoxFactory {
    /// Create a [`CommandInfo`] with the correct sox binary and base flags.
    #[must_use]
    pub(crate) fn create(&self) -> CommandInfo {
        CommandInfo {
            program: self.binary().to_owned(),
            args: self.base_args(),
        }
    }

    /// Return the sox binary name for display purposes.
    #[must_use]
    pub(crate) fn binary(&self) -> &str {
        if self.options.no_sox_ng { SOX } else { SOX_NG }
    }

    fn base_args(&self) -> Vec<String> {
        if self.options.no_sox_ng {
            Vec::new()
        } else {
            vec!["--single-threaded".to_owned()]
        }
    }
}
