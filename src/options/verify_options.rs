use std::fmt::{Display, Formatter};

use clap::{ArgAction, Args};
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::cli::ArgumentsParser;
use crate::cli::CommandArguments::*;
use crate::options::{Options, OptionsProvider};

/// Options for [`VerifyCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct VerifyOptions {
    /// Should the hash check of source files be skipped?
    ///
    /// Note: This is only useful for development and should probably not be used.
    ///
    /// Default: `false`
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub no_hash_check: Option<bool>,
}

#[injectable]
impl VerifyOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for VerifyOptions {
    fn get_name() -> String {
        "Verify Options".to_owned()
    }

    fn merge(&mut self, alternative: &Self) {
        if self.no_hash_check.is_none() {
            self.no_hash_check = alternative.no_hash_check;
        }
    }

    fn apply_defaults(&mut self) {
        if self.no_hash_check.is_none() {
            self.no_hash_check = Some(false);
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        true
    }

    #[must_use]
    fn from_args() -> Option<Self> {
        let options = match ArgumentsParser::get() {
            Some(Batch { verify, .. }) => verify,
            Some(Verify { verify, .. }) => verify,
            _ => return None,
        };
        let mut options = options;
        if options.no_hash_check == Some(false) {
            options.no_hash_check = None;
        }
        Some(options)
    }

    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }
}

impl Display for VerifyOptions {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(json) = serde_json::to_string_pretty(self) {
            json
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
