//! Verify a FLAC source is suitable for transcoding.

pub(crate) use stream_verifier::*;
pub(crate) use tag_verifier::*;
pub(crate) use verify_command::*;
pub(crate) use verify_status::*;

mod stream_verifier;
mod tag_verifier;
#[cfg(test)]
mod tests;
mod verify_command;
mod verify_status;
