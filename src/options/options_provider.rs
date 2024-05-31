use std::path::PathBuf;

use colored::Colorize;
use di::injectable;
use log::*;

use crate::logging::{Logger, Trace};
use crate::options::*;

const DEFAULT_CONFIG_PATH: &str = "config.json";

/// Retrieve options
///
/// Options are retrieved from multiple sources, and merged in order of precedence:
/// 1. Command line arguments
/// 2. Config file defined by the command line argument
/// 3. `config.json` in the current working directory
pub struct OptionsProvider {
    json: Option<String>,
}

#[injectable]
impl OptionsProvider {
    #[must_use]
    pub fn new() -> Self {
        let cli_options = Arguments::get_shared_options().unwrap_or_default();
        Self {
            json: Some(read_config_file(&cli_options)),
        }
    }

    /// Get the [`SharedOptions`]
    #[must_use]
    pub fn get_shared_options(&self) -> SharedOptions {
        let options = Arguments::get_shared_options();
        self.get(options)
    }

    /// Get the [`SpectrogramOptions`]
    #[must_use]
    pub fn get_spectrogram_options(&self) -> SpectrogramOptions {
        let options = Arguments::get_spectrogram_options();
        self.get(options)
    }

    /// Get the [`TranscodeOptions`]
    #[must_use]
    pub fn get_transcode_options(&self) -> TranscodeOptions {
        let options = Arguments::get_transcode_options();
        // clap seems to ignore None as a default_value so we have to manually set to None
        // or file options will be ignored
        if let Some(options) = options {
            let mut options = options;
            if options.allow_existing == Some(false) {
                options.allow_existing = None;
            }
            if options.skip_hash_check == Some(false) {
                options.skip_hash_check = None;
            }
            if options.hard_link == Some(false) {
                options.hard_link = None;
            }
            if options.compress_images == Some(false) {
                options.compress_images = None;
            }
            self.get(Some(options))
        } else {
            self.get(options)
        }
    }

    /// Get the [`Options`]
    #[must_use]
    pub fn get<T: Options>(&self, options: Option<T>) -> T {
        let mut options = if let Some(options) = options {
            trace!(
                "{} {} from command line:\n{}",
                "Parsed".bold(),
                T::get_name(),
                options
            );
            options
        } else {
            warn!(
                "{} to parse {} from command line.",
                "Failed".bold(),
                T::get_name(),
            );
            T::default()
        };
        if let Some(json) = &self.json {
            match T::from_json(json) {
                Ok(file_options) => {
                    trace!(
                        "{} {} from file:\n{}",
                        "Parsed".bold(),
                        T::get_name(),
                        file_options
                    );
                    options.merge(&file_options);
                }
                Err(error) => {
                    force_init_logger();
                    Logger::init_new(Trace);
                    error!(
                        "{} to deserialize config file: {}",
                        "Failed".bold().red(),
                        error
                    );
                }
            }
        }
        options.apply_defaults();
        debug!("{} {}: {}", "Using".bold(), T::get_name(), options);
        options
    }
}

/// [`SharedOptions`] are read before [`Logger`] is initialized so if an error occurs
/// it will be lost to the void unless we force inititialization.
fn force_init_logger() {
    Logger::init_new(Trace);
}

/// Read the config file
///
/// Use the default config path if no path is set on the command line.
fn read_config_file(options: &SharedOptions) -> String {
    let path = options
        .config_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    trace!("{} options from file: {:?}", "Reading".bold(), path);
    match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            force_init_logger();
            warn!(
                "{} to read config file: {}",
                "Failed".bold().yellow(),
                error
            );
            "{}".to_owned()
        }
    }
}
