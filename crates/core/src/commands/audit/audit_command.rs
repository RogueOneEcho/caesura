use crate::prelude::*;
use std::fs::canonicalize;

const MAX_DEPTH: usize = 10;

/// Scan a directory of `.torrent` files for problematic file paths.
#[injectable]
pub(crate) struct AuditCommand {
    args: Ref<AuditArgs>,
    options: Ref<AuditOptions>,
    auditor: Ref<TorrentAuditor>,
}

impl AuditCommand {
    /// Execute [`AuditCommand`] from the CLI.
    ///
    /// The scan directory is retrieved from the CLI arguments.
    pub(crate) fn execute_cli(&self) -> Result<bool, Failure<AuditAction>> {
        let path = self
            .args
            .audit_path
            .clone()
            .expect("audit path should be set after validation");
        let summary = self.execute(&path)?;
        debug!("{} {} torrent files", "Audited".bold(), summary.total);
        if summary.issues.is_empty() {
            info!("No issues found");
            return Ok(true);
        }
        warn!(
            "{} {} problematic torrents",
            "Found".bold(),
            summary.issues.len()
        );
        for item in &summary.issues {
            let Some(issues) = &item.issues else {
                continue;
            };
            eprintln!(
                "{}",
                format!(
                    "{} issues with {}",
                    issues.len(),
                    item.render(self.options.print_bb_code)
                )
                .bold()
            );
            for issue in issues {
                eprintln!("{}", issue.render(self.options.print_bb_code));
            }
        }
        eprintln!("\n{}", summary.kind_table());
        Ok(false)
    }

    /// Scan `path` for problematic torrents.
    ///
    /// - Lists `.torrent` files in the directory
    /// - Inspects each file
    pub(crate) fn execute(&self, input: &Path) -> Result<AuditSummary, Failure<AuditAction>> {
        let mut items = Vec::new();
        let paths = get_paths(input)?;
        for path in &paths {
            let item = self.auditor.execute_path(path);
            if item.issues.is_some() {
                items.push(item);
            }
        }
        Ok(AuditSummary {
            total: paths.len(),
            issues: items,
        })
    }
}

fn get_paths(path: &Path) -> Result<Vec<PathBuf>, Failure<AuditAction>> {
    let path =
        canonicalize(path).map_err(Failure::wrap_with_path(AuditAction::Canonicalize, path))?;
    if path.is_file() {
        return Ok(vec![path.clone()]);
    }
    DirectoryReader::new()
        .with_extension("torrent")
        .with_max_depth(MAX_DEPTH)
        .read(&path)
        .map_err(Failure::wrap_with_path(AuditAction::ReadDirectory, &path))
}

#[cfg(test)]
impl AuditCommand {
    /// Create an [`AuditCommand`] that reports every problem for testing.
    pub(crate) fn mock() -> Self {
        Self {
            args: Ref::new(AuditArgs { audit_path: None }),
            auditor: Ref::new(TorrentAuditor::mock()),
            options: Ref::new(AuditOptions {
                print_bb_code: true,
                ..AuditOptions::default()
            }),
        }
    }
}

/// Error action for [`AuditCommand`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum AuditAction {
    #[error("canonicalize path")]
    Canonicalize,
    #[error("read directory")]
    ReadDirectory,
}
