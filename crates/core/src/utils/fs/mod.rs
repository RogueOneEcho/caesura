//! File system operations, FLAC file handling, and path management.

pub(crate) use additional_file::*;
pub(crate) use collector::*;
pub(crate) use copy_dir::*;
pub(crate) use directory_reader::*;
pub(crate) use flac_file::*;
pub(crate) use fs_action::*;
pub(crate) use path_manager::*;
pub(crate) use tags::*;
pub(crate) use tags_action::*;

mod additional_file;
mod collector;
mod copy_dir;
mod directory_reader;
mod flac_file;
mod fs_action;
mod path_manager;
mod tags;
mod tags_action;
#[cfg(test)]
mod tests;
