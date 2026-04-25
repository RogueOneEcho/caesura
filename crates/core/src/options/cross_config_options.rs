use crate::prelude::*;

/// Validation label for the cross config file.
const CROSS_CONFIG_LABEL: &str = "Cross Config File";

/// Options pointing to the cross indexer's configuration.
///
/// Used by the `cross` command for cross-seed lookups.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct CrossConfigOptions {
    /// Path to a config file for the cross indexer.
    ///
    /// Only `api_key`, `indexer`, and `indexer_url` are used.
    #[arg(long)]
    pub cross_config: Option<PathBuf>,
}

impl CrossConfigOptions {
    /// Load and resolve the cross indexer's [`SharedOptions`] from the config file.
    ///
    /// - Returns `OptionRule::NotSet` if `cross_config` is not set.
    /// - Returns `OptionRule::DoesNotExist` if the path is not a file.
    /// - Returns `OptionRule::ConfigDeserialize` if the file cannot be read or parsed.
    pub fn load_shared_options(&self) -> Result<SharedOptions, OptionRule> {
        let Some(path) = &self.cross_config else {
            return Err(OptionRule::NotSet(CROSS_CONFIG_LABEL.to_owned()));
        };
        let path = path.expand_tilde();
        if !path.is_file() {
            return Err(OptionRule::DoesNotExist(
                CROSS_CONFIG_LABEL.to_owned(),
                path.to_string_lossy().to_string(),
            ));
        }
        let yaml =
            read_to_string(&path).map_err(|e| OptionRule::ConfigDeserialize(e.to_string()))?;
        let partial: SharedOptionsPartial =
            yaml_from_str(&yaml).map_err(|e| OptionRule::ConfigDeserialize(e.to_string()))?;
        Ok(partial.resolve_without_validation())
    }
}

impl OptionsContract for CrossConfigOptions {
    type Partial = CrossConfigOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if self.cross_config.is_none() {
            return;
        }
        let options = match self.load_shared_options() {
            Ok(options) => options,
            Err(e) => {
                errors.push(e);
                return;
            }
        };
        if options.api_key.is_empty() {
            errors.push(OptionRule::NotSet("Cross API key".to_owned()));
        }
        if options.indexer_url.is_empty() {
            errors.push(OptionRule::NotSet("Cross indexer URL".to_owned()));
        }
        if options.indexer_url.ends_with('/') {
            errors.push(OptionRule::UrlInvalidSuffix(
                "Cross indexer URL".to_owned(),
                options.indexer_url.clone(),
            ));
        }
        if !options.indexer_url.starts_with("https://")
            && !options.indexer_url.starts_with("http://")
        {
            errors.push(OptionRule::UrlNotHttp(
                "Cross indexer URL".to_owned(),
                options.indexer_url,
            ));
        }
    }
}
