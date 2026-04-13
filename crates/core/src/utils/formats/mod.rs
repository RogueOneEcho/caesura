//! Audio format types and conversion utilities.

pub(crate) use edition_key::*;
pub(crate) use existing_format::*;
pub(crate) use existing_format_provider::*;
pub(crate) use source_format::*;
pub(crate) use target_format::*;
pub(crate) use target_format_provider::*;

mod edition_key;
mod existing_format;
mod existing_format_provider;
mod source_format;
mod target_format;
mod target_format_provider;
#[cfg(test)]
mod tests;
