use serde::{Deserialize, Serialize};

use caesura_macros::Options;
use caesura_options::{OptionRule, OptionsContract};

/// Options for copying files
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct CopyOptions {
    /// Should files be hard linked instead of copied?
    ///
    /// Enabling this option requires the source and destination to be on the same filesystem or mounted volume.
    #[arg(long)]
    pub hard_link: bool,
}

impl OptionsContract for CopyOptions {
    type Partial = CopyOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
