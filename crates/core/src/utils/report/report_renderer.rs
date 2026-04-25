use crate::prelude::*;
use colored::control;
use lofty::tag::ItemKey;

/// Render markdown reports for reportable source issues.
#[injectable]
pub(crate) struct ReportRenderer {
    shared_options: Ref<SharedOptions>,
}

/// Restores `colored` control override on drop.
struct OverrideGuard;

impl Drop for OverrideGuard {
    fn drop(&mut self) {
        control::unset_override();
    }
}

impl ReportRenderer {
    /// Render a markdown report for a source with reportable issues.
    ///
    /// - Caller must filter to reportable issues first ([`SourceIssue::is_reportable`])
    pub(crate) fn render(
        &self,
        source: &Source,
        issues: &[SourceIssue],
    ) -> Result<String, Failure<ReportAction>> {
        control::set_override(false);
        let _guard = OverrideGuard;

        let inspect = if source.directory.is_dir() {
            Some(
                InspectFactory::new(false)
                    .create(&source.directory)
                    .map_err(Failure::wrap(ReportAction::InspectFiles))?,
            )
        } else {
            None
        };
        let mut out = String::new();
        self.write_all(&mut out, source, issues, inspect.as_deref())
            .expect("writing to a String is infallible");
        Ok(out)
    }

    fn write_all(
        &self,
        out: &mut String,
        source: &Source,
        issues: &[SourceIssue],
        inspect: Option<&str>,
    ) -> FmtResult {
        self.write_header(out, source)?;
        write_detected_issues(out, issues)?;
        write_suggested_reports(out, source, issues)?;
        write_body(out, source, issues, inspect)?;
        Ok(())
    }

    fn write_header(&self, out: &mut String, source: &Source) -> FmtResult {
        let title = SourceName::get_unsanitized(&source.metadata);
        let permalink = get_permalink(
            &self.shared_options.indexer_url,
            source.group.id,
            source.torrent.id,
        );
        let report_url = get_report_url(&self.shared_options.indexer_url, source.torrent.id);
        writeln!(out, "# {title}")?;
        writeln!(out)?;
        writeln!(out, "- **Source:** {permalink}")?;
        writeln!(out, "- **Report:** {report_url}")?;
        writeln!(out)?;
        Ok(())
    }
}

fn write_detected_issues(out: &mut String, issues: &[SourceIssue]) -> FmtResult {
    writeln!(out, "## Detected issues")?;
    writeln!(out)?;
    for issue in issues {
        writeln!(out, "- {issue}")?;
    }
    writeln!(out)?;
    Ok(())
}

fn write_suggested_reports(out: &mut String, source: &Source, issues: &[SourceIssue]) -> FmtResult {
    writeln!(out, "## Suggested reports")?;
    writeln!(out)?;
    let mut by_type: Vec<(&'static str, Vec<&SourceIssue>)> = Vec::new();
    for issue in issues {
        if let Some(report_type) = issue.report_type() {
            if let Some(entry) = by_type.iter_mut().find(|(key, _)| *key == report_type) {
                entry.1.push(issue);
            } else {
                by_type.push((report_type, vec![issue]));
            }
        }
    }
    for (report_type, matched) in &by_type {
        match format_track_numbers(source, matched) {
            Some(line) => {
                writeln!(out, "- **{report_type}** - Track Number(s): `{line}`")?;
            }
            None => {
                writeln!(out, "- **{report_type}**")?;
            }
        }
    }
    writeln!(out)?;
    Ok(())
}

fn write_body(
    out: &mut String,
    source: &Source,
    issues: &[SourceIssue],
    inspect: Option<&str>,
) -> FmtResult {
    writeln!(out, "## Report body")?;
    writeln!(out)?;
    writeln!(
        out,
        "Paste the block below into the \"Comments\" field of the report form."
    )?;
    writeln!(out)?;
    writeln!(out, "```")?;
    for issue in issues {
        writeln!(out, "{}:", issue.report_label())?;
        for path in issue.affected_paths() {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |file_name| file_name.to_string_lossy().into_owned(),
            );
            writeln!(out, "- \"{name}\"")?;
        }
        writeln!(out)?;
    }
    let dir_name = source.directory.file_name().map_or_else(
        || source.directory.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    );
    writeln!(out, "[code]caesura inspect \"{dir_name}\"[/code]")?;
    writeln!(out)?;
    writeln!(out, "[pre]")?;
    if let Some(inspect) = inspect {
        writeln!(out, "{}", inspect.trim_end())?;
    }
    writeln!(out, "[/pre]")?;
    writeln!(out, "```")?;
    Ok(())
}

/// Build a space-separated track-number list for a suggested report entry.
///
/// - Returns `Some("all")` when every FLAC in the source is affected
/// - Returns `Some(numbers)` when every affected file has a parseable numeric `TRACKNUMBER`
/// - Returns `None` otherwise
fn format_track_numbers(source: &Source, issues: &[&SourceIssue]) -> Option<String> {
    if !source.directory.is_dir() {
        return None;
    }
    let flacs = Collector::get_flacs_with_context(&source.directory);
    if flacs.is_empty() {
        return None;
    }
    let affected: BTreeSet<&Path> = issues
        .iter()
        .flat_map(|issue| issue.affected_paths().into_iter())
        .collect();
    if affected.len() == flacs.len() {
        return Some(String::from("all"));
    }
    let mut numbers: Vec<u32> = Vec::new();
    for path in &affected {
        let flac = flacs.iter().find(|flac| flac.path == **path)?;
        let tags = flac.id3_tags().ok()?;
        let track = tags.get_string(ItemKey::TrackNumber)?;
        let number: u32 = u32::from_str(track).ok()?;
        numbers.push(number);
    }
    numbers.sort_unstable();
    numbers.dedup();
    Some(
        numbers
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    )
}
