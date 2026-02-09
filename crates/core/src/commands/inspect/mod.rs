//! Inspect audio file metadata.

pub(crate) use inspect_action::*;
pub(crate) use inspect_arg::*;
pub(crate) use inspect_command::*;

mod format;
mod inspect_action;
mod inspect_arg;
mod inspect_command;
mod picture_info;
mod track_info;

#[cfg(test)]
mod tests;
