use std::env::var;

use colored::Colorize;
use log::{debug, warn};

use crate::options::{Options, OptionsProvider, SharedOptions};

pub struct TestOptionsFactory;

impl TestOptionsFactory {
    #[must_use]
    pub fn from_with_env(options: SharedOptions) -> SharedOptions {
        let provider = OptionsProvider::new();
        let mut options = inject_from_env_var(options);
        options.merge(&provider.get());
        options
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
    if options.announce_url.is_none() {
        options.announce_url = get_env_var("ANNOUNCE_URL");
    }
    options
}

fn get_env_var(key: &str) -> Option<String> {
    if let Ok(value) = var(key) {
        debug!("{} {key} from environment variable", "Assigning".bold());
        Some(value)
    } else {
        warn!("Environment variable {} is not set", key.bold());
        None
    }
}
