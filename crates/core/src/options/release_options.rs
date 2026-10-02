//! Options for checking GitHub for newer releases.

use crate::prelude::*;

/// Options for the [`ReleaseProvider`].
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct ReleaseOptions {
    /// Skip checking GitHub for a newer release.
    #[arg(long)]
    pub offline: bool,
}

impl OptionsContract for ReleaseOptions {
    type Partial = ReleaseOptionsPartial;

    fn validate(&self, _validator: &mut OptionsValidator) {}
}
