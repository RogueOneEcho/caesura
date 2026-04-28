use crate::prelude::*;

/// Render grouped source issues as plain text.
pub(crate) struct SourceIssuesRenderer;

impl SourceIssuesRenderer {
    /// Render issues grouped by type with sorted relative paths.
    ///
    /// - Groups share the same [`SourceIssue::group_key`]
    /// - Files within a group are sorted by their path relative to `source_dir`
    /// - Groups are ordered alphabetically by key
    /// - Issues without an affected path render as a standalone line
    pub(crate) fn render(issues: &[SourceIssue], source_dir: &Path) -> String {
        let groups = group_by_key(issues, source_dir);
        let mut out = String::new();
        Self::write(&mut out, &groups).expect("writing to a String is infallible");
        out
    }

    fn write(out: &mut String, groups: &[(String, BTreeSet<PathBuf>)]) -> FmtResult {
        for (i, (key, paths)) in groups.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if paths.is_empty() {
                writeln!(out, "{key}")?;
            } else {
                writeln!(out, "{key}:")?;
                for path in paths {
                    writeln!(out, "- \"{}\"", path.display())?;
                }
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn mock(source_dir: &Path) -> Vec<SourceIssue> {
        vec![
            SourceIssue::UnnecessaryDirectory {
                prefix: source_dir.to_path_buf(),
            },
            SourceIssue::MissingTags {
                path: source_dir.join("05. Track.flac"),
                tags: vec!["track number".to_owned(), "artist".to_owned()],
            },
            SourceIssue::MissingTags {
                path: source_dir.join("03. Track.flac"),
                tags: vec!["artist".to_owned()],
            },
            SourceIssue::MissingTags {
                path: source_dir.join("01. Track.flac"),
                tags: vec!["track number".to_owned(), "artist".to_owned()],
            },
            SourceIssue::FlacError {
                path: source_dir.join("01. Track.flac"),
                error: "bad header".to_owned(),
            },
            SourceIssue::MissingTags {
                path: source_dir.join("08. Track.flac"),
                tags: vec!["track number".to_owned(), "artist".to_owned()],
            },
            SourceIssue::MissingTags {
                path: source_dir.join("10. Track.flac"),
                tags: vec!["artist".to_owned()],
            },
            SourceIssue::FlacError {
                path: source_dir.join("05. Track.flac"),
                error: "bad header".to_owned(),
            },
            SourceIssue::FlacError {
                path: source_dir.join("03. Track.flac"),
                error: "bad header".to_owned(),
            },
            SourceIssue::MissingTags {
                path: source_dir.join("CD2/01. Disk 2 track.flac"),
                tags: vec!["album".to_owned()],
            },
            SourceIssue::MissingTags {
                path: source_dir.join("CD2/03. Disk 2 track.flac"),
                tags: vec!["album".to_owned()],
            },
            SourceIssue::MissingTags {
                path: source_dir.join("CD1/01. Track.flac"),
                tags: vec!["album".to_owned()],
            },
            SourceIssue::NoTags {
                path: source_dir.join("CD1/01. Track.flac"),
            },
        ]
    }
}

/// Group issues by [`SourceIssue::group_key`] with deduplicated relative paths.
///
/// - Strips `source_dir` from each affected path
/// - Deduplicates and sorts paths within each group
/// - Sorts groups alphabetically by key
fn group_by_key(issues: &[SourceIssue], source_dir: &Path) -> Vec<(String, BTreeSet<PathBuf>)> {
    let mut groups: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();
    for issue in issues {
        let paths = issue
            .affected_paths()
            .into_iter()
            .map(|p| p.strip_prefix(source_dir).unwrap_or(p).to_path_buf());
        groups
            .entry(issue.render(PathStyle::None))
            .or_default()
            .extend(paths);
    }
    groups.into_iter().collect()
}
