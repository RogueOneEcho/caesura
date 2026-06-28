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

    /// Should the decode test of each FLAC be skipped?
    ///
    /// By default every audio frame is decoded to detect truncation or corruption.
    #[arg(long)]
    pub no_decode_test: bool,
}

impl OptionsContract for VerifyOptions {
    type Partial = VerifyOptionsPartial;
    fn validate(&self, _validator: &mut OptionsValidator) {}
}
