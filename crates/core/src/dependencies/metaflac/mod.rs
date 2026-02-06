//! Facade for the `metaflac` FLAC metadata CLI.

pub(crate) use metaflac_action::*;
pub(crate) use metaflac_command::*;

mod metaflac_action;
mod metaflac_command;
