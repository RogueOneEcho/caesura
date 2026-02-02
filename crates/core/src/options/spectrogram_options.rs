use crate::Size;
use crate::commands::CommandArguments::{self, *};
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for spectrograms
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct SpectrogramOptions {
    /// Sizes of spectrograms to generate.
    #[arg(long)]
    #[options(default = vec![Size::Full, Size::Zoom])]
    pub spectrogram_size: Vec<Size>,
}

impl OptionsContract for SpectrogramOptions {
    type Partial = SpectrogramOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if self.spectrogram_size.is_empty() {
            errors.push(IsEmpty("Spectrogram Size".to_owned()));
        }
    }
}

impl FromArgs for SpectrogramOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { spectrogram, .. } | Spectrogram { spectrogram, .. }) => {
                Some(spectrogram.clone())
            }
            _ => None,
        }
    }
}
