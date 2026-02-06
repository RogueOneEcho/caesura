//! Display current configuration options.

pub(crate) use config_action::*;
pub(crate) use config_command::*;

mod config_action;
mod config_command;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
