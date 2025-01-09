use crate::commands::*;
use crate::options::*;
use std::fmt::{Display, Formatter};

use clap::{ArgAction, Args};
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::*;

/// Options for copying files during [`TranscodeCommand`] and [`UploadCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct CopyOptions {
    /// Should files be hard linked instead of copied?
    ///
    /// Enabling this option requires the source and destination to be on the same filesystem or mounted volume.
    ///
    /// Default: `false`
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub hard_link: Option<bool>,
}

#[injectable]
impl CopyOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for CopyOptions {
    fn get_name() -> String {
        "Copy Options".to_owned()
    }

    fn merge(&mut self, alternative: &Self) {
        if self.hard_link.is_none() {
            self.hard_link = alternative.hard_link;
        }
    }

    fn apply_defaults(&mut self) {
        if self.hard_link.is_none() {
            self.hard_link = Some(false);
        }
    }

    #[must_use]
    fn validate(&self) -> bool {
        true
    }

    #[must_use]
    fn from_args() -> Option<Self> {
        let Some(Batch { copy, .. } | Transcode { copy, .. } | Upload { copy, .. }) =
            ArgumentsParser::get()
        else {
            return None;
        };
        let mut options = copy;
        if options.hard_link == Some(false) {
            options.hard_link = None;
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

impl Display for CopyOptions {
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
