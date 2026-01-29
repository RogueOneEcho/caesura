//! Directory and file naming conventions for transcodes and spectrograms.

pub(crate) use disc_context::*;
pub(crate) use humanize::*;
pub(crate) use sanitizer::*;
pub(crate) use shortener::*;
pub(crate) use source_name::*;
pub(crate) use spectrogram_name::*;
pub(crate) use transcode_name::*;

mod disc_context;
mod humanize;
mod sanitizer;
mod shortener;
mod source_name;
mod spectrogram_name;
#[cfg(test)]
mod tests;
mod track_name;
mod transcode_name;
