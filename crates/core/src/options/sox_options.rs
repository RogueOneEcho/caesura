//! Options for selecting the sox binary variant.

use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{self, *};
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;

/// Options for sox binary selection
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SoxOptions {
    /// Use the original `SoX` binary instead of SoX-ng.
    ///
    /// When set, the binary name changes from `sox_ng` to `sox` and the
    /// `--single-threaded` flag is omitted.
    #[arg(long)]
    pub no_sox_ng: bool,
}

impl OptionsContract for SoxOptions {
    type Partial = SoxOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}

impl FromArgs for SoxOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(
                Batch { sox, .. }
                | Spectrogram { sox, .. }
                | Transcode { sox, .. }
                | Version { sox, .. },
            ) => Some(sox.clone()),
            _ => None,
        }
    }
}
