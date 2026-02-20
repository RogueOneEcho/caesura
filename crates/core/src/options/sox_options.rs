//! Options for selecting the sox binary variant.

use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{self, *};
use crate::dependencies::{DETECTED_SOX_VARIANT, SoxVariant};
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;

/// Options for sox binary selection
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SoxOptions {
    /// `SoX` binary to use.
    ///
    /// Options: `sox` or `sox_ng`
    #[arg(long, value_enum)]
    #[options(default_fn = default_sox_variant, default_doc = "auto-detected")]
    pub sox_variant: SoxVariant,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_sox_variant(_partial: &SoxOptionsPartial) -> Option<SoxVariant> {
    Some(*DETECTED_SOX_VARIANT)
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
