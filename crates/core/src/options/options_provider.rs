use crate::options::*;
use colored::Colorize;
use di::injectable;
use log::*;
use rogue_logging::Verbosity::Trace;
use rogue_logging::{Logger, LoggerBuilder};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;

/// Retrieve options
///
/// Options are retrieved from multiple sources, and merged in order of precedence:
/// 1. Command line arguments
/// 2. Config file defined by the `--config` command line argument
/// 3. `config.yml` in the current working directory
pub struct OptionsProvider {
    yaml: Option<String>,
}

#[injectable]
impl OptionsProvider {
    #[must_use]
    pub fn new() -> Self {
        let cli_options = SharedOptions::from_args().unwrap_or_default();
        Self {
            yaml: Some(read_config_file(&cli_options)),
        }
    }

    /// Get the [`Options`]
    #[must_use]
    pub fn get<T: Options>(&self) -> T {
        let mut options = T::from_args().unwrap_or_default();
        if let Some(yaml) = &self.yaml
            && !yaml.is_empty()
        {
            match T::from_yaml(yaml) {
                Ok(file_options) => {
                    options.merge(&file_options);
                }
                Err(error) => {
                    let _ = init_logger();
                    error!("{} to deserialize config file: {}", "Failed".bold(), error);
                }
            }
        }
        options.apply_defaults();
        options
    }
}

/// Read the config file
///
/// Use the default config path if no path is set on the command line.
fn read_config_file(options: &SharedOptions) -> String {
    let path = options
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    read_to_string(path).unwrap_or_else(|error| {
        let _ = init_logger();
        warn!("{} to read config file: {}", "Failed".bold(), error);
        "{}".to_owned()
    })
}

fn init_logger() -> Arc<Logger> {
    LoggerBuilder::new()
        .with_exclude_filter("reqwest".to_owned())
        .with_exclude_filter("cookie".to_owned())
        .with_verbosity(Trace)
        .create()
}
