//! Factory for building sox commands with the correct binary and flags.

use std::process::Command;
use std::sync::LazyLock;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Factory for creating sox [`CommandInfo`] with the correct binary and flags.
///
/// Uses the configured [`SoxVariant`] variant to select the binary and base arguments.
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
        self.options.sox_variant.binary()
    }

    fn base_args(&self) -> Vec<String> {
        match self.options.sox_variant {
            SoxVariant::Sox => Vec::new(),
            SoxVariant::SoxNg => vec!["--single-threaded".to_owned()],
        }
    }
}

/// Sox binary variant.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum SoxVariant {
    Sox,
    #[serde(alias = "ng", alias = "soxng", alias = "sox-ng")]
    #[value(alias = "ng", alias = "soxng", alias = "sox-ng")]
    #[default]
    SoxNg,
}

impl SoxVariant {
    /// Binary name for this variant.
    #[must_use]
    pub(crate) fn binary(self) -> &'static str {
        match self {
            Self::Sox => SOX,
            Self::SoxNg => SOX_NG,
        }
    }
}

/// Auto-detected sox variant, determined once by checking if `sox_ng` is available.
pub(crate) static DETECTED_SOX_VARIANT: LazyLock<SoxVariant> = LazyLock::new(|| {
    if Command::new("sox_ng").arg("--version").output().is_ok() {
        SoxVariant::SoxNg
    } else {
        SoxVariant::Sox
    }
});
