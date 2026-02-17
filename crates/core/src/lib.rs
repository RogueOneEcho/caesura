//! CLI tool for transcoding FLAC audio and uploading to Gazelle-based music trackers.

#[allow(unused_imports)]
pub use commands::*;
pub use hosting::*;
pub use utils::logging::*;
pub mod options;

mod commands;
mod dependencies;
mod hosting;
mod prelude;
#[cfg(test)]
mod testing_prelude;
mod utils;
