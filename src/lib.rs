pub mod api;
pub mod cli;
pub mod dependencies;
pub mod errors;
pub mod formats;
pub mod fs;
pub mod hosting;
pub mod imdl;
pub mod jobs;
pub mod logging;
pub mod naming;
pub mod options;
pub mod source;
pub mod spectrogram;
pub mod testing;
pub mod transcode;
pub mod verify;

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
