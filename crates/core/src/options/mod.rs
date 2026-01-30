//! Configuration structs loaded from CLI args and YAML config.

pub use batch_options::*;
pub use cache_options::*;
pub(crate) use command::*;
pub use copy_options::*;
pub use doc_metadata::*;
pub use file_options::*;
pub(crate) use options_provider::*;
pub(crate) use options_trait::{ApplicableCommands, OptionsPartial};
pub(crate) use queue_add_args::*;
pub(crate) use queue_rm_args::*;
pub(crate) use rules::OptionRule::*;
pub(crate) use rules::*;
pub(crate) use runner_options::*;
pub(crate) use shared_options::*;
pub(crate) use source_arg::*;
pub use spectrogram_options::*;
pub use target_options::*;
pub use upload_options::*;
pub use verify_options::*;

mod batch_options;
mod cache_options;
mod command;
mod copy_options;
mod doc_metadata;
mod file_options;
mod options_provider;
mod options_trait;
mod queue_add_args;
mod queue_rm_args;
mod rules;
mod runner_options;
mod shared_options;
mod source_arg;
mod spectrogram_options;
mod target_options;
#[cfg(test)]
mod tests;
mod upload_options;
mod verify_options;
