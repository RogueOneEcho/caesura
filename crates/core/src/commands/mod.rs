//! CLI command implementations.

pub use arguments_parser::*;
pub(crate) use batch::*;
pub(crate) use command_arguments::*;
pub(crate) use config::*;
pub(crate) use docs::*;
pub(crate) use inspect::*;
pub(crate) use queue::*;
pub(crate) use spectrogram::*;
pub(crate) use test_name::*;
pub(crate) use transcode::*;
pub(crate) use upload::*;
pub(crate) use verify::*;
pub(crate) use version::*;

mod arguments_parser;
mod batch;
mod command_arguments;
mod config;
mod docs;
mod inspect;
mod queue;
mod spectrogram;
mod test_name;
mod transcode;
mod upload;
mod verify;
mod version;
