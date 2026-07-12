use crate::prelude::*;
use std::fs::canonicalize;

const MAX_DEPTH: usize = 10;

/// Scan `.torrent` files for problematic file paths.
#[injectable]
pub(crate) struct AuditCommand {
    args: Ref<AuditArgs>,
    options: Ref<AuditOptions>,
    auditor: Ref<TorrentAuditor>,
    torrent_file_provider: Ref<TorrentFileProvider>,
    shared: Ref<SharedOptions>,
    cache: Ref<CacheOptions>,
}

impl AuditCommand {
    /// Execute [`AuditCommand`] from the CLI.
    ///
    /// - A numeric input is downloaded from the API and audited
    /// - Any other input is scanned as a file or directory path
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<AuditAction>> {
        let mode = self.args.to_mode().expect("arg should be valid");
        let paths = match mode {
            AuditMode::Directory(path) => get_dir_paths(&path)?,
            AuditMode::File(path) => vec![path],
            AuditMode::Id(id) => vec![self.download_by_id(id).await?],
        };
        let summary = self.execute(&paths);
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
        self.print_issues(&summary);
        Ok(false)
    }

    /// Scan `paths` for problematic torrents.
    pub(crate) fn execute(&self, paths: &[PathBuf]) -> AuditSummary {
        let mut items = Vec::new();
        for path in paths {
            let item = self.auditor.execute_path(path);
            if item.issues.is_some() {
                items.push(item);
            }
        }
        AuditSummary {
            total: paths.len(),
            issues: items,
        }
    }

    /// Download the torrent for `id` from the API.
    ///
    /// - Validates the API credentials and cache directory before downloading
    async fn download_by_id(&self, id: u32) -> Result<PathBuf, Failure<AuditAction>> {
        self.validate_id_inputs()?;
        self.torrent_file_provider
            .get(id)
            .await
            .map_err(Failure::wrap(AuditAction::Download))
    }

    /// Validate the credentials and cache directory the id flow uses.
    ///
    /// - Checks `api_key` is set, `indexer_url` is a valid URL, and the cache directory exists
    /// - Logs each issue via [`OptionsValidator::check`] before returning an error
    fn validate_id_inputs(&self) -> Result<(), Failure<AuditAction>> {
        let mut validator = OptionsValidator::new();
        if self.shared.api_key.is_empty() {
            validator.push(OptionIssue::required_non_empty("api_key"));
        }
        validator.check_url("indexer_url", &self.shared.indexer_url);
        self.cache.validate(&mut validator);
        if validator.check() {
            Ok(())
        } else {
            Err(Failure::from_action(AuditAction::ValidateOptions))
        }
    }

    fn print_issues(&self, summary: &AuditSummary) {
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
    }
}

fn get_dir_paths(path: &Path) -> Result<Vec<PathBuf>, Failure<AuditAction>> {
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

/// Error action for [`AuditCommand`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum AuditAction {
    #[error("canonicalize path")]
    Canonicalize,
    #[error("download torrent")]
    Download,
    #[error("read directory")]
    ReadDirectory,
    #[error("validate options")]
    ValidateOptions,
}
