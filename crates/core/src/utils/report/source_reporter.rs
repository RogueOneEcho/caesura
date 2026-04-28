use crate::prelude::*;
use std::fs::write as fs_write;

/// Write automatically generated tracker reports for reportable issues.
#[injectable]
pub(crate) struct SourceReporter {
    report_options: Ref<ReportOptions>,
    verify_options: Ref<VerifyOptions>,
    shared_options: Ref<SharedOptions>,
    renderer: Ref<ReportRenderer>,
}

impl SourceReporter {
    /// Write a report for `source` if the issues are reportable and the
    /// source content is verified.
    pub(crate) fn execute(
        &self,
        source: &Source,
        issues: &[SourceIssue],
    ) -> Result<(), Failure<ReportAction>> {
        let Some(reportable) = self.get_reportable(issues) else {
            return Ok(());
        };
        let body = self.renderer.render(source, &reportable)?;
        let path = self.get_report_path(source);
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .map_err(Failure::wrap_with_path(ReportAction::CreateDir, parent))?;
        }
        fs_write(&path, body).map_err(Failure::wrap_with_path(ReportAction::WriteFile, &path))?;
        info!("{} written to {}", "Report".bold(), path.display());
        Ok(())
    }

    fn get_report_path(&self, source: &Source) -> PathBuf {
        let indexer = self.shared_options.get_indexer();
        let filename = format!("{}-{}.md", indexer.as_lowercase(), source.torrent.id);
        self.report_options.reports_dir.join(filename)
    }

    fn get_reportable(&self, issues: &[SourceIssue]) -> Option<Vec<SourceIssue>> {
        if self.report_options.no_reports {
            trace!("Skipping report: disabled");
            return None;
        }
        if self.verify_options.no_hash_check {
            trace!("Skipping report: hash check disabled");
            return None;
        }
        if let Some(blocker) = issues.iter().find(|issue| blocks_report(issue)) {
            trace!("Skipping report: {}", blocker.render(PathStyle::Plain));
            return None;
        }
        let reportable: Vec<SourceIssue> = issues
            .iter()
            .filter(|issue| issue.is_reportable())
            .cloned()
            .collect();
        if reportable.is_empty() {
            trace!("{} no reportable issues", "Report skipped:".bold());
            return None;
        }
        Some(reportable)
    }
}

fn blocks_report(issue: &SourceIssue) -> bool {
    matches!(
        issue,
        SourceIssue::HashCheck { .. }
            | SourceIssue::MissingFile { .. }
            | SourceIssue::OpenFile { .. }
            | SourceIssue::ExcessContent
            | SourceIssue::MissingDirectory { .. }
            | SourceIssue::NoFlacs { .. }
            | SourceIssue::FlacCount { .. }
            | SourceIssue::Trumpable
    )
}
