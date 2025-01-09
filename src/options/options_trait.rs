use std::fmt::Display;

pub trait Options: Clone + Default + Display {
    /// Get a friendly display name.
    #[allow(dead_code)]
    fn get_name() -> String;

    /// Merge values with [`Self`]
    fn merge(&mut self, alternative: &Self);

    /// Apply default values to [`Self`]
    fn apply_defaults(&mut self);

    /// Validate [`Self`]
    fn validate(&self) -> bool;

    /// Get [`Self`] from the command line arguments
    fn from_args() -> Option<Self>;

    /// Deserialize [`Self`] from JSON
    #[allow(clippy::absolute_paths, dead_code)]
    fn from_json(json: &str) -> Result<Self, serde_json::error::Error>;

    /// Deserialize [`Self`] from YAML
    fn from_yaml(json: &str) -> Result<Self, serde_yaml::Error>;
}
