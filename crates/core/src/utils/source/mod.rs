//! Source identification, metadata extraction, and validation.

pub(crate) use id_provider::*;
pub(crate) use metadata::*;
pub(crate) use source::*;
pub(crate) use source_action::*;
pub(crate) use source_issue::*;
pub(crate) use source_provider::SourceProvider;
pub(crate) use url_helpers::*;

mod id_provider;
mod metadata;
mod source;
mod source_action;
mod source_issue;
mod source_provider;
mod status_helpers;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
mod url_helpers;
