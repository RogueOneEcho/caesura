use crate::Size;
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

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
