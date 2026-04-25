//! Find a source on the primary indexer and look up cross-seeds on the cross indexer.

pub(crate) use cross_command::*;

mod cross_command;
#[cfg(test)]
mod tests;
