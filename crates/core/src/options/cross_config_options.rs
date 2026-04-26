use crate::prelude::*;

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
    /// - Returns [`OptionIssue::required`] if `cross_config` is not set.
    /// - Returns [`OptionIssue::file_not_found`] if the path is not a file.
    /// - Returns [`OptionIssue::config_invalid`] if the file cannot be read or parsed.
    pub fn load_shared_options(&self) -> Result<SharedOptions, OptionIssue> {
        let Some(path) = &self.cross_config else {
            return Err(OptionIssue::required("cross_config"));
        };
        let path = path.expand_tilde();
        if !path.is_file() {
            return Err(OptionIssue::file_not_found("cross_config", &path));
        }
        let yaml =
            read_to_string(&path).map_err(|e| OptionIssue::config_invalid(&e.to_string()))?;
        let partial: SharedOptionsPartial =
            yaml_from_str(&yaml).map_err(|e| OptionIssue::config_invalid(&e.to_string()))?;
        Ok(partial.resolve_without_validation())
    }
}

impl OptionsContract for CrossConfigOptions {
    type Partial = CrossConfigOptionsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        if self.cross_config.is_none() {
            return;
        }
        let options = match self.load_shared_options() {
            Ok(options) => options,
            Err(e) => {
                validator.push(e);
                return;
            }
        };
        if options.api_key.is_empty() {
            validator.push(OptionIssue::config_invalid(
                "cross indexer api_key is empty",
            ));
        }
        if options.indexer_url.is_empty() {
            validator.push(OptionIssue::config_invalid(
                "cross indexer indexer_url is empty",
            ));
        } else if !options.indexer_url.starts_with("http://")
            && !options.indexer_url.starts_with("https://")
        {
            validator.push(OptionIssue::config_invalid(&format!(
                "cross indexer indexer_url must start with http:// or https://: {}",
                options.indexer_url
            )));
        } else if options.indexer_url.ends_with('/') {
            validator.push(OptionIssue::config_invalid(&format!(
                "cross indexer indexer_url must not end with a trailing slash: {}",
                options.indexer_url
            )));
        }
    }
}
