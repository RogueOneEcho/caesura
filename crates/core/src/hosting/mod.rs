//! Dependency injection container setup and CLI host.

pub(crate) use host::*;
pub use host_builder::*;

mod host;
mod host_builder;
