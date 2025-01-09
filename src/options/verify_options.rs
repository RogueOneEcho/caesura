use std::fmt::{Display, Formatter};

use clap::{ArgAction, Args};
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::commands::*;
use crate::options::*;

use crate::commands::CommandArguments::*;

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

    /// Should sources with specific tags be excluded?
    ///
    /// Default: None
    #[arg(long)]
    pub exclude_tags: Option<Vec<String>>,
}

#[injectable]
impl VerifyOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for VerifyOptions {
    fn merge(&mut self, alternative: &Self) {
        if self.no_hash_check.is_none() {
            self.no_hash_check = alternative.no_hash_check;
        }
        if self.exclude_tags.is_none() {
            self.exclude_tags.clone_from(&alternative.exclude_tags);
        }
    }

    fn apply_defaults(&mut self) {
        if self.no_hash_check.is_none() {
            self.no_hash_check = Some(false);
        }
        if self.exclude_tags.is_none() {
            self.exclude_tags = Some(Vec::new());
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        true
    }

    #[must_use]
    fn from_args() -> Option<Self> {
        let Some(Batch { verify, .. } | Verify { verify, .. }) = ArgumentsParser::get() else {
            return None;
        };
        let mut options = verify;
        if options.no_hash_check == Some(false) {
            options.no_hash_check = None;
        }
        Some(options)
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for VerifyOptions {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(yaml) = serde_yaml::to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
