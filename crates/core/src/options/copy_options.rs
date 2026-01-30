use serde::{Deserialize, Serialize};

use crate::options::OptionRule;
use caesura_macros::Options;

/// Options for copying files
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Transcode, Upload))]
#[options(field_name = "copy")]
#[derive(Default)]
pub struct CopyOptions {
    /// Should files be hard linked instead of copied?
    ///
    /// Enabling this option requires the source and destination to be on the same filesystem or mounted volume.
    ///
    /// Default: `false`
    #[arg(long)]
    pub hard_link: bool,
}

impl CopyOptions {
    /// Validate the partial options.
    pub fn validate_partial(_: &CopyOptionsPartial, _: &mut Vec<OptionRule>) {}
}
