use crate::options::{
    Options, OptionsProvider, SharedOptions, SpectrogramOptions, TranscodeOptions,
};
use crate::testing::CONTENT_SAMPLES_DIR;
use colored::Colorize;
use log::{debug, warn};
use std::env::var;
use std::path::PathBuf;

pub struct TestOptionsFactory;

impl TestOptionsFactory {
    #[must_use]
    pub fn shared(mut options: SharedOptions) -> SharedOptions {
        let provider = OptionsProvider::new();
        options.merge(&provider.get_shared_options());
        inject_from_env_var(options)
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

fn inject_from_env_var(options: SharedOptions) -> SharedOptions {
    let mut options = options;
    if options.api_key.is_none() {
        options.api_key = get_env_var("API_KEY");
    }
    if options.source.is_none() {
        options.source = get_env_var("SOURCE");
    }
    if options.content_directory.is_none() {
        options.content_directory = Some(PathBuf::from(CONTENT_SAMPLES_DIR));
    }
    options
}

fn get_env_var(key: &str) -> Option<String> {
    if let Ok(value) = var(key) {
        debug!("{} {key} from environment variable", "Assigning".bold());
        Some(value)
    } else {
        warn!("Environment variable {} is not set", key.bold().yellow());
        None
    }
}
