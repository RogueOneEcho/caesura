pub use hosting::*;
mod commands;
mod dependencies;
mod hosting;
mod options;
mod utils;

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
