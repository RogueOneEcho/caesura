use serde::{Deserialize, Serialize};

use crate::options::OptionRule;
use caesura_macros::Options;

/// Options for verify
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Verify))]
#[options(field_name = "verify")]
#[derive(Default)]
pub struct VerifyOptions {
    /// Should the hash check of source files be skipped?
    ///
    /// Note: This is only useful for development and should probably not be used.
    ///
    /// Default: `false`
    #[arg(long)]
    pub no_hash_check: bool,

    /// Should sources with specific tags be excluded?
    ///
    /// Default: None
    #[arg(long)]
    pub exclude_tags: Option<Vec<String>>,
}

impl VerifyOptions {
    /// Validate the partial options.
    pub fn validate_partial(_: &VerifyOptionsPartial, _: &mut Vec<OptionRule>) {}
}
