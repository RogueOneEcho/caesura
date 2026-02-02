use crate::commands::CommandArguments::{self, *};
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for transcoding
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct TargetOptions {
    /// Formats to attempt to transcode to.
    #[arg(long)]
    #[options(default = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0])]
    pub target: Vec<TargetFormat>,

    /// Allow transcoding to existing formats.
    ///
    /// Note: This is only useful for development and should probably not be used.
    #[arg(long)]
    pub allow_existing: bool,

    /// Use random dithering when resampling with `SoX`.
    ///
    /// By default, `SoX` runs in repeatable mode (`-R`) which seeds the dither
    /// random number generator with a fixed value, producing deterministic output.
    /// Set this to `true` to use random dithering instead.
    #[arg(long)]
    pub sox_random_dither: bool,
}

impl OptionsContract for TargetOptions {
    type Partial = TargetOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if self.target.is_empty() {
            errors.push(IsEmpty("Target format".to_owned()));
        }
    }
}

impl FromArgs for TargetOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(
                Batch { target, .. }
                | Transcode { target, .. }
                | Upload { target, .. }
                | Verify { target, .. },
            ) => Some(target.clone()),
            _ => None,
        }
    }
}
