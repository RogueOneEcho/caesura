//! Dependency injection container setup and CLI host.

mod build_error;
mod host;
mod host_builder;
#[cfg(test)]
mod host_builder_testing;
mod shutdown;

pub use build_error::*;
pub(crate) use host::*;
pub use host_builder::*;
pub use shutdown::*;
