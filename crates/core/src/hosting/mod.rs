//! Dependency injection container setup and CLI host.

mod build_error;
mod host;
mod host_builder;

pub(crate) use host::*;
pub use build_error::*;
pub use host_builder::*;
