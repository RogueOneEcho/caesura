use crate::prelude::*;

/// Summary of an [`AuditCommand`] run.
pub(crate) struct AuditSummary {
    /// Number of torrent files inspected this run.
    pub total: usize,
    /// Number of inspected torrents with problems.
    pub issues: Vec<AuditItem>,
}

impl AuditSummary {
    /// Render a table summarizing how many torrents carry each issue kind.
    ///
    /// - Counts each torrent once per kind, even with repeated issues
    /// - Sorts by torrent count descending, then by kind name ascending
    /// - Percentages are of the total torrents checked
    pub(crate) fn kind_table(&self) -> String {
        let mut counts: HashMap<AuditIssueKind, usize> = HashMap::new();
        for item in &self.issues {
            let Some(issues) = &item.issues else {
                continue;
            };
            let kinds: HashSet<AuditIssueKind> = issues.iter().map(|issue| issue.kind).collect();
            for kind in kinds {
                *counts.entry(kind).or_default() += 1;
            }
        }
        let mut rows: Vec<(AuditIssueKind, usize)> = counts.into_iter().collect();
        rows.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_string().cmp(&b.0.to_string()))
        });
        let mut builder = TableBuilder::new()
            .headers(["Issue", "Torrents", "%"])
            .right_align(vec![false, true, true]);
        for (kind, count) in rows {
            let percent = percent(count, self.total);
            builder = builder.row([kind.to_string(), count.to_string(), percent]);
        }
        builder.build()
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "percentage rounding"
)]
fn percent(value: usize, total: usize) -> String {
    let percent = value as f64 / total as f64 * 100.0;
    format!("{percent:.2}%")
}
