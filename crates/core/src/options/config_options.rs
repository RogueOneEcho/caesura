use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Legacy config path from before platform user directories.
const LEGACY_CONFIG_PATH: &str = "./config.yml";

/// Configuration file path.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct ConfigOptions {
    /// Path to the configuration file.
    #[arg(long)]
    #[options(default_doc = "`~/.config/caesura/config.yml` or platform equivalent")]
    pub config: Option<PathBuf>,
}

impl OptionsContract for ConfigOptions {
    type Partial = ConfigOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(config) = &self.config
            && !config.is_file()
        {
            errors.push(DoesNotExist(
                "Config File".to_owned(),
                config.to_string_lossy().to_string(),
            ));
        }
        if !self
            .config
            .clone()
            .unwrap_or(PathManager::default_config_path())
            .is_file()
            && PathBuf::from(LEGACY_CONFIG_PATH).is_file()
        {
            let default_path = PathManager::default_config_path();
            errors.push(Changed(
                "Config File".to_owned(),
                default_path.to_string_lossy().to_string(),
                format!("In v0.27.0 the default config path changed to {}.\nPass the option: --config {LEGACY_CONFIG_PATH} to use the previous config path.", default_path.display()),
            ));
        }
    }
}
