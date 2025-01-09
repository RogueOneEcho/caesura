use std::collections::BTreeSet;

use di::{injectable, Ref};

use crate::options::*;
use crate::utils::*;

#[injectable]
pub struct TargetFormatProvider {
    pub options: Ref<TargetOptions>,
}

impl TargetFormatProvider {
    /// Get the target formats for a [Source].
    #[must_use]
    pub fn get(
        &self,
        source: SourceFormat,
        existing: &BTreeSet<ExistingFormat>,
    ) -> BTreeSet<TargetFormat> {
        if self.options.allow_existing == Some(true) {
            self.get_with_existing(source)
        } else {
            self.get_without_existing(existing)
        }
    }

    /// Get the target format with the longest path length.
    ///
    /// `FLAC` + `.flac` = 9 characters
    /// `320` + `.mp3` = 7 characters
    /// `V0` + `.mp3` = 6 characters
    pub fn get_max_path_length(
        &self,
        source: SourceFormat,
        existing: &BTreeSet<ExistingFormat>,
    ) -> Option<TargetFormat> {
        self.get(source, existing).first().copied()
    }

    /// Filter the target formats to exclude the source format.
    fn get_with_existing(&self, source: SourceFormat) -> BTreeSet<TargetFormat> {
        let set = BTreeSet::from([source.to_existing()]);
        self.get_targets_except_excluded(&set)
    }

    /// Filter the target formats to exclude existing formats (which will include the source format).
    fn get_without_existing(&self, existing: &BTreeSet<ExistingFormat>) -> BTreeSet<TargetFormat> {
        self.get_targets_except_excluded(existing)
    }

    fn get_targets_except_excluded(
        &self,
        exclude: &BTreeSet<ExistingFormat>,
    ) -> BTreeSet<TargetFormat> {
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
