use std::fmt::Display;

/// Common interface for configuration option structs.
pub trait Options: Clone + Default + Display {
    /// Merge values with [`Self`]
    fn merge(&mut self, alternative: &Self);

    /// Apply default values to [`Self`]
    fn apply_defaults(&mut self);

    /// Validate [`Self`]
    fn validate(&self) -> bool;

    /// Get [`Self`] from the command line arguments
    fn from_args() -> Option<Self>;

    /// Deserialize [`Self`] from YAML
    fn from_yaml(json: &str) -> Result<Self, serde_yaml::Error>;
}
