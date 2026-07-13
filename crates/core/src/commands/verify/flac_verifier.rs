use crate::prelude::*;

/// Verify a [`Source`]'s FLAC files are suitable for transcoding.
#[injectable]
pub(crate) struct FlacVerifier {
    paths: Ref<PathManager>,
}

impl FlacVerifier {
    /// Run FLAC-level checks over a pre-collected, non-empty list of FLAC.
    ///
    /// Assumes the directory exists and the list is non-empty; those cases are
    /// handled by [`Collector::collect_flacs`] before this is called.
    pub(crate) fn execute(
        &self,
        source: &Source,
        flacs: &[FlacFile],
    ) -> Result<Vec<SourceIssue>, Failure<VerifyAction>> {
        trace!("{} {} FLACs", "Checking".bold(), flacs.len());
        let mut issues: Vec<SourceIssue> = Vec::new();
        issues.extend(check_flac_count(source, flacs.len()));
        issues.extend(check_subdirectory(flacs));
        let max_target = get_max_path_length_target(source);
        let output_dir = self.paths.get_output_dir();
        for flac in flacs {
            if let Some(max_target) = max_target {
                let path = self
                    .paths
                    .get_transcode_path(source, max_target, flac)
                    .strip_prefix(output_dir.clone())
                    .expect("should be able to strip prefix from transcode path")
                    .to_path_buf();
                issues.extend(check_path_length(&path));
            }
            let tag_issues = TagVerifier::execute(flac, source)
                .map_err(Failure::wrap(VerifyAction::VerifyTags))?;
            issues.extend(tag_issues);
            issues.extend(StreamVerifier::execute(flac, source));
        }
        Ok(issues)
    }
}

/// Check the FLAC file count matches the torrent metadata.
pub(crate) fn check_flac_count(source: &Source, actual: usize) -> Option<SourceIssue> {
    let expected = source.torrent.get_flacs().len();
    if actual != expected {
        return Some(SourceIssue::FlacCount { expected, actual });
    }
    None
}

/// Check whether all FLAC files share an unnecessary common subdirectory prefix.
///
/// - If every FLAC sits under one common subdirectory of the torrent root,
///   that directory is unnecessary and trumpable.
/// - Multi-disc sets separate items by subdirectory, so they have no common prefix.
/// - Targets the common case where a single unnecessary directory holds all FLAC
///   content, likely due to a misunderstanding of how the creation tool works.
pub(crate) fn check_subdirectory(flacs: &[FlacFile]) -> Option<SourceIssue> {
    let flac_sub_dirs: Vec<_> = flacs.iter().map(|x| &x.sub_dir).collect();
    Shortener::longest_common_prefix(&flac_sub_dirs)
        .map(|prefix| SourceIssue::UnnecessaryDirectory { prefix })
}

/// Get the target format with the longest path length.
///
/// - `FLAC` + `.flac` = 9 characters
/// - `320` + `.mp3` = 7 characters
/// - `V0` + `.mp3` = 6 characters
///
/// [`BTreeSet<TargetFormat>`] is ordered by discriminant value so the first
/// element is always the format with the longest path.
fn get_max_path_length_target(source: &Source) -> Option<TargetFormat> {
    source.targets.first().copied()
}

/// Check the transcode path length does not exceed the maximum.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::as_conversions,
    reason = "bounded path length"
)]
pub(crate) fn check_path_length(path: &Path) -> Option<SourceIssue> {
    let length = path.to_string_lossy().chars().count() as isize;
    let excess = length - MAX_PATH_LENGTH;
    if excess > 0 {
        return Some(SourceIssue::Length {
            path: path.to_path_buf(),
            excess: excess as usize,
        });
    }
    None
}
