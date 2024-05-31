use crate::options::{
    Options, OptionsProvider, SharedOptions, SpectrogramOptions, TranscodeOptions,
};

pub struct TestOptionsFactory;

impl TestOptionsFactory {
    #[must_use]
    pub fn shared(mut options: SharedOptions) -> SharedOptions {
        let provider = OptionsProvider::new();
        options.merge(&provider.get_shared_options());
        options
    }

    #[must_use]
    pub fn spectrogram(mut options: SpectrogramOptions) -> SpectrogramOptions {
        let provider = OptionsProvider::new();
        options.merge(&provider.get_spectrogram_options());
        options
    }

    #[must_use]
    pub fn transcode(mut options: TranscodeOptions) -> TranscodeOptions {
        let provider = OptionsProvider::new();
        options.merge(&provider.get_transcode_options());
        options
    }
}
