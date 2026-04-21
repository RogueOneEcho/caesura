//! Options for configuring the sox binary path and feature set.

use crate::prelude::*;
use std::process::Command;

/// Options for sox binary selection.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SoxOptions {
    /// Name or path to the sox binary.
    ///
    /// Examples: `sox`, `sox_ng`, `/usr/bin/sox`
    #[arg(long)]
    #[options(default_fn = default_sox_path, default_doc = "Detected based on sox_ng flag")]
    pub sox_path: Option<PathBuf>,

    /// Is `SoX_ng` in use?
    ///
    /// If `true` then `sox_ng` specific CLI options are used.
    #[arg(long)]
    #[options(default_fn = default_sox_ng, default_doc = "Detected based on binary name or --version info")]
    pub sox_ng: bool,
}

fn default_sox_path(_partial: &SoxOptionsPartial) -> Option<PathBuf> {
    if is_docker() {
        return Some(PathBuf::from("sox_ng"));
    }
    None
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_sox_ng(partial: &SoxOptionsPartial) -> Option<bool> {
    if is_docker() {
        return Some(true);
    }
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

/// Cached result of `sox_ng` detection.
///
/// Spawns `sox --version` once per process and caches the result.
static IS_SOX_NG: LazyLock<bool> = LazyLock::new(|| {
    let Ok(output) = Command::new(SOX).arg("--version").output() else {
        return true;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("SoX_ng")
});

/// Detect whether the `sox` binary on PATH is actually `sox_ng`.
///
/// - Returns `true` if `sox` is not found (assumes `sox_ng` is installed separately).
/// - Returns `true` if `sox --version` identifies itself as `sox_ng`.
pub(crate) fn detect_sox_ng() -> bool {
    *IS_SOX_NG
}
