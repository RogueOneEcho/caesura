use std::fmt::{Display, Formatter};

use clap::Args;
use di::{Ref, injectable};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{Batch, Spectrogram};
use crate::commands::*;
use crate::options::*;
/// Options for [`SpectrogramCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpectrogramOptions {
    /// Sizes of spectrograms to generate.
    ///
    /// Default: `full` and `zoom`
    #[arg(long)]
    pub spectrogram_size: Option<Vec<Size>>,
}

#[injectable]
impl SpectrogramOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for SpectrogramOptions {
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

    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
        let size = self.spectrogram_size.as_ref();
        if size.is_none() || size.is_some_and(Vec::is_empty) {
            errors.push(IsEmpty("Spectrogram Size".to_owned()));
        }
        OptionRule::show(&errors);
        errors.is_empty()
    }

    fn from_args() -> Option<SpectrogramOptions> {
        match ArgumentsParser::get() {
            Some(Batch { spectrogram, .. } | Spectrogram { spectrogram, .. }) => Some(spectrogram),
            _ => None,
        }
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for SpectrogramOptions {
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
