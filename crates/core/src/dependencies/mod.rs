//! Facades for external CLI binaries invoked as subprocesses.

pub(crate) use binaries::*;
pub(crate) use sox_factory::*;

mod binaries;
mod sox_factory;
