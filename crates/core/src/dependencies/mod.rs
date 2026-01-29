//! Facades for external CLI binaries invoked as subprocesses.

pub(crate) use binaries::*;
pub(crate) use eyed3::*;
pub(crate) use imdl::*;
pub(crate) use metaflac::*;

mod binaries;
mod eyed3;
mod imdl;
mod metaflac;
