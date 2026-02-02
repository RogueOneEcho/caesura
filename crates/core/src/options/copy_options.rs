use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{self, *};
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;

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

impl FromArgs for CopyOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { copy, .. } | Transcode { copy, .. } | Upload { copy, .. }) => {
                Some(copy.clone())
            }
            _ => None,
        }
    }
}
