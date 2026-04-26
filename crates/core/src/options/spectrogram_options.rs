use crate::Size;
use crate::prelude::*;

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

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_non_empty("spectrogram_size", &self.spectrogram_size);
    }
}
