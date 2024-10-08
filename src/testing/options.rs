use std::env::var;
use std::path::PathBuf;

use colored::Colorize;
use log::{debug, warn};

use crate::options::{Options, OptionsProvider, SharedOptions};
use crate::testing::CONTENT_SAMPLES_DIR;

pub struct TestOptionsFactory;

impl TestOptionsFactory {
    #[must_use]
    pub fn from_with_env(mut options: SharedOptions) -> SharedOptions {
        let provider = OptionsProvider::new();
        options.merge(&provider.get());
        inject_from_env_var(options)
    }

    #[must_use]
    pub fn from<T: Options>(mut options: T) -> T {
        let provider = OptionsProvider::new();
        options.merge(&provider.get());
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
