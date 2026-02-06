//! Batch processing of multiple sources from a queue.

pub(crate) use batch_action::*;
pub(crate) use batch_command::*;

mod batch_action;
mod batch_command;
#[cfg(test)]
mod tests;
