//! Facade for the `imdl` torrent CLI.

pub(crate) use imdl_command::*;
pub(crate) use torrent_summary::*;

mod imdl_command;
mod torrent_summary;
