use crate::prelude::*;

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

impl ConfigOptions {
    /// Config file path with tilde expansion applied.
    #[must_use]
    pub fn path(&self) -> Option<PathBuf> {
        self.config.as_ref().map(ExpandTilde::expand_tilde)
    }
}

impl OptionsContract for ConfigOptions {
    type Partial = ConfigOptionsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        if let Some(config) = self.path()
            && !config.is_file()
        {
            validator.push(OptionIssue::file_not_found("config", &config));
        }
        let resolved = self
            .config
            .clone()
            .unwrap_or_else(PathManager::default_config_path);
        if !resolved.is_file() && PathBuf::from(LEGACY_CONFIG_PATH).is_file() {
            let default_path = PathManager::default_config_path();
            validator.push(OptionIssue::default_changed(
                "config",
                &resolved.to_string_lossy(),
                &format!(
                    "In v0.27.0 the default config path changed to {}.\nPass the option: --config {LEGACY_CONFIG_PATH} to use the previous config path.",
                    default_path.display()
                ),
            ));
        }
    }
}
