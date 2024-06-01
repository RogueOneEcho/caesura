use std::fmt::{Display, Formatter};

use crate::options::{IsEmpty, OptionRule, Options, OptionsProvider};
use crate::spectrogram::Size;
use clap::Args;
use di::{injectable, Ref};
use serde::{Deserialize, Serialize};

/// Options for the [`SpectrogramGenerator`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpectrogramOptions {
    /// Output directory to write spectrogram images to
    #[arg(long)]
    pub spectrogram_size: Option<Vec<Size>>,
}

#[injectable]
impl SpectrogramOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get_spectrogram_options()
    }
}

impl Options for SpectrogramOptions {
    fn get_name() -> String {
        "Spectrogram Options".to_owned()
    }

    /// Merge the current options with an alternative set of options
    ///
    /// The current options will take precedence over the alternative options
    fn merge(&mut self, alternative: &Self) {
        if self.spectrogram_size.is_none() {
            self.spectrogram_size
                .clone_from(&alternative.spectrogram_size);
        }
    }

    fn apply_defaults(&mut self) {
        if self.spectrogram_size.is_none() {
            self.spectrogram_size = Some(vec![Size::Full, Size::Zoom]);
        }
    }

    /// Validate the options
    #[must_use]
    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        let size = self.spectrogram_size.as_ref();
        if size.is_none() || size.is_some_and(Vec::is_empty) {
            errors.push(IsEmpty("Spectrogram Size".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    fn from_json(json: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }
}

impl Display for SpectrogramOptions {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(json) = serde_json::to_string_pretty(self) {
            json
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
