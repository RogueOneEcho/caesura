use serde::{Deserialize, Serialize};

use crate::Size;
use crate::options::*;
use caesura_macros::Options;

/// Options for spectrograms
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Spectrogram))]
#[options(field_name = "spectrogram")]
pub struct SpectrogramOptions {
    /// Sizes of spectrograms to generate.
    ///
    /// Default: `full` and `zoom`
    #[arg(long)]
    #[options(default = vec![Size::Full, Size::Zoom])]
    pub spectrogram_size: Vec<Size>,
}

impl Default for SpectrogramOptions {
    fn default() -> Self {
        Self {
            spectrogram_size: vec![Size::Full, Size::Zoom],
        }
    }
}

impl SpectrogramOptions {
    /// Validate the partial options.
    pub fn validate_partial(partial: &SpectrogramOptionsPartial, errors: &mut Vec<OptionRule>) {
        // Only error if explicitly set to empty (None will use default)
        if partial.spectrogram_size.as_ref().is_some_and(Vec::is_empty) {
            errors.push(IsEmpty("Spectrogram Size".to_owned()));
        }
    }
}
