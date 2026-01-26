#[allow(unused_imports)]
pub use commands::*;
pub use hosting::*;

mod commands;
mod dependencies;
mod hosting;
mod options;
mod utils;

#[allow(clippy::needless_raw_strings, clippy::doc_markdown)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
