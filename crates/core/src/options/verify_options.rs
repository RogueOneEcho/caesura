use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{self, *};
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;

/// Options for verify
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct VerifyOptions {
    /// Should the hash check of source files be skipped?
    ///
    /// Note: This is only useful for development and should probably not be used.
    #[arg(long)]
    pub no_hash_check: bool,

    /// Should sources with specific tags be excluded?
    #[arg(long)]
    pub exclude_tags: Option<Vec<String>>,
}

impl OptionsContract for VerifyOptions {
    type Partial = VerifyOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}

impl FromArgs for VerifyOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { verify, .. } | Verify { verify, .. }) => Some(verify.clone()),
            _ => None,
        }
    }
}
