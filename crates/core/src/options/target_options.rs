use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for transcoding
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Transcode, Upload, Verify))]
#[options(field_name = "target")]
pub struct TargetOptions {
    /// Formats to attempt to transcode to.
    ///
    /// Default: `flac`, `320` and `v0`
    #[arg(long)]
    #[options(default = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0])]
    pub target: Vec<TargetFormat>,

    /// Allow transcoding to existing formats
    ///
    /// Note: This is only useful for development and should probably not be used.
    ///
    /// Default: `false`
    #[arg(long)]
    pub allow_existing: bool,

    /// Use random dithering when resampling with `SoX`.
    ///
    /// By default, `SoX` runs in repeatable mode (`-R`) which seeds the dither
    /// random number generator with a fixed value, producing deterministic output.
    /// Set this to `true` to use random dithering instead.
    ///
    /// Default: `false`
    #[arg(long)]
    pub sox_random_dither: bool,
}

impl Default for TargetOptions {
    fn default() -> Self {
        Self {
            target: vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0],
            allow_existing: false,
            sox_random_dither: false,
        }
    }
}

impl TargetOptions {
    /// Validate the partial options.
    pub fn validate_partial(partial: &TargetOptionsPartial, errors: &mut Vec<OptionRule>) {
        // Only error if explicitly set to empty (None will use default)
        if partial.target.as_ref().is_some_and(Vec::is_empty) {
            errors.push(IsEmpty("Target format".to_owned()));
        }
    }
}
