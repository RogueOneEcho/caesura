use crate::prelude::*;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub(crate) struct VerifyCommand {
    verify_options: Ref<VerifyOptions>,
    source_provider: Ref<SourceProvider>,
    api_verifier: Ref<ApiVerifier>,
    paths: Ref<PathManager>,
    torrents: Ref<TorrentFileProvider>,
    reporter: Ref<SourceReporter>,
}

impl VerifyCommand {
    /// Execute [`VerifyCommand`] from the CLI.
    ///
    /// [`Source`] is retrieved from the CLI arguments.
    ///
    /// [`SourceIssue`] issues are logged as warnings.
    ///
    /// Returns `true` if the source is verified.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<VerifyAction>> {
        let source = match self.source_provider.get_from_options().await {
            Ok(Ok(source)) => source,
            Ok(Err(issue)) => {
                let status = VerifyStatus::from_issue(issue);
                warn!("{} for transcoding unknown", "Unsuitable".bold());
                if let Some(issues) = &status.issues {
                    for issue in issues {
                        warn!("{issue}");
                    }
                }
                return Ok(false);
            }
            Err(e) => return Err(Failure::new(VerifyAction::GetSource, e)),
        };
        let result = self.execute(&source).await?;
        let id = source.to_string();
        if result.verified() {
            info!("{} {id}", "Verified".bold());
        } else {
            let issues = SourceIssuesRenderer::render(&result.issues, &source.directory);
            warn!("{} for transcoding {id}\n{issues}", "Unsuitable".bold());
        }
        Ok(result.verified())
    }

    /// Execute [`VerifyCommand`] on a [`Source`].
    ///
    /// Returns a [`VerifySuccess`] containing any issues found.
    pub(crate) async fn execute(
        &self,
        source: &Source,
    ) -> Result<VerifySuccess, Failure<VerifyAction>> {
        debug!("{} {}", "Verifying".bold(), source);
        let mut issues: Vec<SourceIssue> = Vec::new();
        issues.append(&mut self.api_verifier.execute(source));
        issues.append(&mut self.flac_checks(source)?);
        if let Some(issue) = self.hash_check(source).await? {
            issues.push(issue);
        }
        if let Err(failure) = self.reporter.execute(source, &issues) {
            warn!("{}", failure.render());
        }
        Ok(VerifySuccess { issues })
    }

    fn flac_checks(&self, source: &Source) -> Result<Vec<SourceIssue>, Failure<VerifyAction>> {
        if let Some(issue) = check_directory_exists(source) {
            return Ok(vec![issue]);
        }
        trace!("Collecting FLACs from {}", source.directory.display());
        let flacs = Collector::get_flacs_with_context(&source.directory);
        if flacs.is_empty() {
            return Ok(vec![SourceIssue::NoFlacs {
                path: source.directory.clone(),
            }]);
        }
        let mut issues: Vec<SourceIssue> = Vec::new();
        issues.extend(check_flac_count(source, flacs.len()));
        issues.append(&mut VerifyCommand::subdirectory_checks(&flacs));
        let max_target = get_max_path_length_target(source);
        let output_dir = self.paths.get_output_dir();
        for flac in flacs {
            trace!("Verifying FLAC {}", flac.path.display());
            if let Some(max_target) = max_target {
                let path = self
                    .paths
                    .get_transcode_path(source, max_target, &flac)
                    .strip_prefix(output_dir.clone())
                    .expect("should be able to strip prefix from transcode path")
                    .to_path_buf();
                issues.extend(check_path_length(&path));
            }
            let tag_issue = TagVerifier::execute(&flac, source)
                .map_err(Failure::wrap(VerifyAction::VerifyTags))?;
            issues.extend(tag_issue);
            for error in StreamVerifier::execute(&flac) {
                issues.push(error);
            }
        }
        Ok(issues)
    }

    /// Verify the source files match the torrent hash, unless disabled in options.
    pub(crate) async fn hash_check(
        &self,
        source: &Source,
    ) -> Result<Option<SourceIssue>, Failure<VerifyAction>> {
        if self.verify_options.no_hash_check {
            debug!("{} hash check due to settings", "Skipped".bold());
            return Ok(None);
        }
        let torrent_path = self.get_source_torrent(source).await?;
        trace!(
            "Verifying torrent hash against {}",
            source.directory.display()
        );
        TorrentVerifier::execute(&torrent_path, &source.directory)
            .await
            .map_err(Failure::wrap(VerifyAction::VerifyHash))
    }

    /// Retrieve the source `.torrent` file, downloading from the API if not cached.
    pub(crate) async fn get_source_torrent(
        &self,
        source: &Source,
    ) -> Result<PathBuf, Failure<VerifyAction>> {
        trace!("Fetching torrent file for {}", source.torrent.id);
        self.torrents
            .get(source.torrent.id)
            .await
            .map_err(Failure::wrap(VerifyAction::GetSourceTorrent))
    }

    /// Check whether all FLAC files share an unnecessary common subdirectory prefix.
    pub fn subdirectory_checks(flacs: &[FlacFile]) -> Vec<SourceIssue> {
        // source.directory is the root directory of the torrent. If all flacs share a subdirectory
        // within that, it is unnecessary and trumpable. Multi-disc sets may separate items by
        // subdirs, so they will not be a common prefix.
        // Note that this is meant to verify the most common case, where a single unnecessary
        // directory contains all flac content, likely due to a misunderstanding of how the
        // creation tool works.
        let flac_sub_dirs: Vec<_> = flacs.iter().map(|x| &x.sub_dir).collect();
        if let Some(prefix) = Shortener::longest_common_prefix(&flac_sub_dirs) {
            return vec![SourceIssue::UnnecessaryDirectory { prefix }];
        }
        vec![]
    }
}

/// Check the source directory exists.
pub(crate) fn check_directory_exists(source: &Source) -> Option<SourceIssue> {
    if !source.directory.is_dir() {
        return Some(SourceIssue::MissingDirectory {
            path: source.directory.clone(),
        });
    }
    None
}

/// Check the FLAC file count matches the torrent metadata.
pub(crate) fn check_flac_count(source: &Source, actual: usize) -> Option<SourceIssue> {
    let expected = source.torrent.get_flacs().len();
    if actual != expected {
        return Some(SourceIssue::FlacCount { expected, actual });
    }
    None
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
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::as_conversions
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
