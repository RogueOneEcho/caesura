//! Configuration structs loaded from CLI args and YAML config.

pub(crate) use batch_options::*;
pub(crate) use cache_options::*;
pub(crate) use copy_options::*;
pub(crate) use file_options::*;
pub(crate) use options_provider::*;
pub(crate) use options_trait::*;
pub(crate) use queue_add_args::*;
pub(crate) use queue_rm_args::*;
pub(crate) use rules::OptionRule::*;
pub(crate) use rules::*;
pub(crate) use runner_options::*;
pub(crate) use shared_options::*;
pub(crate) use source_arg::*;
pub(crate) use spectrogram_options::*;
pub(crate) use target_options::*;
pub(crate) use upload_options::*;
pub(crate) use verify_options::*;

mod batch_options;
mod cache_options;
mod copy_options;
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
