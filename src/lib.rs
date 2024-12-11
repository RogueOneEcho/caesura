mod batch;
pub mod cli;
mod dependencies;
mod errors;
mod eyed3;
mod formats;
mod fs;
pub mod hosting;
mod imdl;
mod jobs;
mod naming;
mod options;
mod queue;
mod source;
mod spectrogram;
#[cfg(test)]
mod testing;
mod transcode;
mod upload;
mod verify;

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
