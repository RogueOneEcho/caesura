//! Options for configuring the sox binary path and feature set.

use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::prelude::SOX;
use caesura_macros::Options;
use caesura_options::{OptionRule, OptionsContract};

/// Options for sox binary selection.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SoxOptions {
    /// Custom path to the sox binary.
    ///
    /// When not set, uses `sox_ng` or `sox` from PATH based on the `sox_ng` flag.
    #[arg(long)]
    #[options(default_doc = "auto-detected")]
    pub sox_path: Option<PathBuf>,

    /// Whether to use `sox_ng` behavior (`--single-threaded` flag).
    #[arg(long)]
    #[options(default_fn = default_sox_ng, default_doc = "auto-detected")]
    pub sox_ng: bool,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_sox_ng(partial: &SoxOptionsPartial) -> Option<bool> {
    if let Some(sox_path) = &partial.sox_path {
        let file_name = sox_path.file_name().unwrap_or_default().to_string_lossy();
        if file_name.contains("sox_ng") {
            return Some(true);
        }
    }
    Some(detect_sox_ng())
}

impl OptionsContract for SoxOptions {
    type Partial = SoxOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}

/// Detect whether the `sox` binary on PATH is actually `sox_ng`.
///
/// - Returns `true` if `sox` is not found (assumes `sox_ng` is installed separately).
/// - Returns `true` if `sox --version` identifies itself as `sox_ng`.
pub(crate) fn detect_sox_ng() -> bool {
    let Ok(output) = Command::new(SOX).arg("--version").output() else {
        return true;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.starts_with("sox_ng")
}
