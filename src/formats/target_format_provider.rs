use std::collections::HashSet;

use di::{injectable, Ref};

use crate::formats::{ExistingFormat, SourceFormat, TargetFormat};
use crate::options::TranscodeOptions;

#[injectable]
pub struct TargetFormatProvider {
    pub options: Ref<TranscodeOptions>,
}

impl TargetFormatProvider {
    /// Get the target formats for a [Source].
    #[must_use]
    pub fn get(
        &self,
        source: SourceFormat,
        existing: &HashSet<ExistingFormat>,
    ) -> Vec<TargetFormat> {
        if self.options.allow_existing == Some(true) {
            self.get_with_existing(source)
        } else {
            self.get_without_existing(existing)
        }
    }

    /// Filter the target formats to exclude the source format.
    fn get_with_existing(&self, source: SourceFormat) -> Vec<TargetFormat> {
        let override_existing = [source.to_existing()];
        self.get_targets_except_excluded(&HashSet::from(override_existing))
    }

    /// Filter the target formats to exclude existing formats (which will include the source format).
    fn get_without_existing(&self, existing: &HashSet<ExistingFormat>) -> Vec<TargetFormat> {
        self.get_targets_except_excluded(existing)
    }

    fn get_targets_except_excluded(&self, exclude: &HashSet<ExistingFormat>) -> Vec<TargetFormat> {
        self.options
            .target
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|&target| !exclude.contains(&target.to_existing()))
            .copied()
            .collect()
    }
}
