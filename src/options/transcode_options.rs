use std::fmt::{Display, Formatter};

use clap::{ArgAction, Args};
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

use crate::formats::TargetFormat;
use crate::options::{IsEmpty, NotSet, OptionRule, Options, OptionsProvider};

/// Options for [Transcoder] and [`SourceVerifier`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct TranscodeOptions {
    /// Target formats.
    /// Default: flac, 320, and v0
    #[arg(long)]
    pub target: Option<Vec<TargetFormat>>,

    /// Allow transcoding to existing formats
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub allow_existing: Option<bool>,

    /// Should the torrent hash check of existing files be skipped?
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub skip_hash_check: Option<bool>,

    /// Use hard links when copying files
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub hard_link: Option<bool>,

    /// Should images greater than 750 KB be compressed?
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub compress_images: Option<bool>,
}

#[injectable]
impl TranscodeOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get_transcode_options()
    }
}

impl Options for TranscodeOptions {
    fn get_name() -> String {
        "Transcode Options".to_owned()
    }

    /// Merge the current options with an alternative set of options
    ///
    /// The current options will take precedence over the alternative options
    fn merge(&mut self, alternative: &Self) {
        if self.target.is_none() {
            self.target.clone_from(&alternative.target);
        }
        if self.allow_existing.is_none() {
            self.allow_existing = alternative.allow_existing;
        }
        if self.skip_hash_check.is_none() {
            self.skip_hash_check = alternative.skip_hash_check;
        }
        if self.hard_link.is_none() {
            self.hard_link = alternative.hard_link;
        }
        if self.compress_images.is_none() {
            self.compress_images = alternative.compress_images;
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
        if self.skip_hash_check.is_none() {
            self.skip_hash_check = Some(false);
        }
        if self.hard_link.is_none() {
            self.hard_link = Some(false);
        }
        if self.compress_images.is_none() {
            self.compress_images = Some(false);
        }
    }

    /// Validate the options
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

    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }
}

impl Display for TranscodeOptions {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(json) = serde_json::to_string_pretty(self) {
            json
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
