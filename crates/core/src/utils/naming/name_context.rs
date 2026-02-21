//! Template context for output naming.

use crate::prelude::*;
use serde::Serialize;

/// Template context combining album metadata with resolution-time fields.
#[derive(Serialize)]
pub(crate) struct NameContext {
    /// Album metadata from the API.
    #[serde(flatten)]
    pub(crate) metadata: Metadata,
    /// Target format name (e.g. "FLAC", "320", "V0").
    pub(crate) format: Option<String>,
    /// Static name override from `--name`, if set.
    pub(crate) name: Option<String>,
    /// Is this a spectrogram output?
    pub(crate) spectrogram: bool,
}

impl NameContext {
    /// Create a [`NameContext`] with placeholder values for validation and testing.
    #[must_use]
    pub(crate) fn mock() -> Self {
        Self {
            metadata: Metadata::mock(),
            format: Some("FLAC".to_owned()),
            name: None,
            spectrogram: false,
        }
    }
}
