//! Configuration structs loaded from CLI args and YAML config.

pub use batch_options::*;
pub use cache_options::*;
pub use config_options::*;
pub use copy_options::*;
pub use file_options::*;
pub use publish_seeding_options::*;
pub(crate) use queue_add_args::*;
pub(crate) use queue_rm_args::*;
pub(crate) use runner_options::*;
pub(crate) use shared_options::*;
pub(crate) use source_arg::*;
pub(crate) use sox_options::*;
pub use spectrogram_options::*;
pub use target_options::*;
pub use torrent_injection_options::*;
pub use upload_options::*;
pub use verify_options::*;

mod batch_options;
mod cache_options;
mod config_options;
mod copy_options;
mod file_options;
mod publish_seeding_options;
mod queue_add_args;
mod queue_rm_args;
mod runner_options;
mod shared_options;
mod source_arg;
mod sox_options;
mod spectrogram_options;
mod target_options;
#[cfg(test)]
mod tests;
mod torrent_injection_options;
mod upload_options;
mod verify_options;
