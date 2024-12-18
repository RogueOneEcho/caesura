use std::fmt::{Display, Formatter};

use clap::{ArgAction, Args};
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::cli::ArgumentsParser;
use crate::cli::CommandArguments::*;
use crate::formats::TargetFormat;
use crate::options::{IsEmpty, NotSet, OptionRule, Options, OptionsProvider};

/// Options for [`TranscodeCommand`] and [`VerifyCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct TargetOptions {
    /// Formats to attempt to transcode to.
    ///
    /// Default: `flac`, `320` and `v0`
    #[arg(long)]
    pub target: Option<Vec<TargetFormat>>,

    /// Allow transcoding to existing formats
    ///
    /// Note: This is only useful for development and should probably not be used.
    ///
    /// Default: `false`
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub allow_existing: Option<bool>,
}

#[injectable]
impl TargetOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for TargetOptions {
    fn get_name() -> String {
        "Target Options".to_owned()
    }

    fn merge(&mut self, alternative: &Self) {
        if self.target.is_none() {
            self.target.clone_from(&alternative.target);
        }
        if self.allow_existing.is_none() {
            self.allow_existing = alternative.allow_existing;
        }
    }

    fn apply_defaults(&mut self) {
        if self.target.is_none() {
            self.target = Some(vec![
                TargetFormat::Flac,
                TargetFormat::_320,
                TargetFormat::V0,
            ]);
        }
        if self.allow_existing.is_none() {
            self.allow_existing = Some(false);
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        if let Some(targets) = &self.target {
            if targets.is_empty() {
                errors.push(IsEmpty("Target format".to_owned()));
            }
        } else {
            errors.push(NotSet("Target format".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    #[must_use]
    fn from_args() -> Option<Self> {
        let Some(
            Batch { target, .. }
            | Transcode { target, .. }
            | Upload { target, .. }
            | Verify { target, .. },
        ) = ArgumentsParser::get()
        else {
            return None;
        };
        let mut options = target;
        if options.allow_existing == Some(false) {
            options.allow_existing = None;
        }
        Some(options)
    }
    #[allow(clippy::absolute_paths)]
    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for TargetOptions {
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
