use crate::prelude::*;

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
