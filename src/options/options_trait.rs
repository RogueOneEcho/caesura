use std::fmt::Display;

pub trait Options: Default + Display {
    fn get_name() -> String;

    fn merge(&mut self, alternative: &Self);

    fn apply_defaults(&mut self);

    fn validate(&self) -> bool;

    fn from_json(json: &str) -> Result<Self, serde_json::error::Error>;
}
