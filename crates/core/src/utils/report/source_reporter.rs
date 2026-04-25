use crate::prelude::*;
use std::fs::write as fs_write;

/// Write automatically generated tracker reports for reportable issues.
#[injectable]
pub(crate) struct SourceReporter {
    report_options: Ref<ReportOptions>,
    shared_options: Ref<SharedOptions>,
    renderer: Ref<ReportRenderer>,
}

impl SourceReporter {
    /// Write a report for `source` if any of `issues` is reportable and
    /// reports are not disabled.
    pub(crate) fn execute(
        &self,
        source: &Source,
        issues: &[SourceIssue],
    ) -> Result<(), Failure<ReportAction>> {
        if self.report_options.no_reports {
            return Ok(());
        }
        let reportable: Vec<SourceIssue> = issues
            .iter()
            .filter(|issue| issue.is_reportable())
            .cloned()
            .collect();
        if reportable.is_empty() {
            return Ok(());
        }
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
}
