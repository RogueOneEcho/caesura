//! Core utilities for error handling, file operations, naming, and format conversion.

pub(crate) use app_info::*;
pub(crate) use diagnostic_ext::*;
pub(crate) use formats::*;
pub(crate) use fs::*;
pub(crate) use jobs::*;
pub(crate) use logging::*;
pub(crate) use naming::*;
pub(crate) use process::*;
pub(crate) use rogue_logging::Failure;
pub(crate) use source::*;
pub(crate) use table::*;
#[cfg(any(test, feature = "demo"))]
pub(crate) use testing::*;
pub(crate) use torrent::*;

mod app_info;
mod diagnostic_ext;
mod formats;
mod fs;
mod jobs;
pub mod logging;
mod naming;
mod process;
mod source;
mod table;
#[cfg(any(test, feature = "demo"))]
#[cfg_attr(all(feature = "demo", not(test)), allow(unused))]
mod testing;
mod torrent;
