use caesura_macros::Options;
use caesura_options::{OptionRule, OptionsContract};
use serde::{Deserialize, Serialize};

/// Options for publish seeding behavior.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct PublishSeedingOptions {
    /// Should source files be moved to the seeding destination?
    ///
    /// If disabled, source files are hard linked into the destination.
    #[arg(long)]
    pub move_source: bool,
}

impl OptionsContract for PublishSeedingOptions {
    type Partial = PublishSeedingOptionsPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
