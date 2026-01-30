//! CLI tool for transcoding FLAC audio and uploading to Gazelle-based music trackers.

#[allow(unused_imports)]
pub use commands::*;
pub use hosting::*;
pub mod options;

mod commands;
mod dependencies;
mod hosting;
mod prelude;
#[cfg(test)]
mod testing_prelude;
mod utils;

#[allow(clippy::needless_raw_strings, clippy::doc_markdown)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
