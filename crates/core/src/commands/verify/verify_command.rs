use crate::prelude::*;
use gazelle_api::GazelleClientTrait;
use std::fs::create_dir;
use tokio::fs::{File, rename};
use tokio::io::AsyncWriteExt;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub(crate) struct VerifyCommand {
    verify_options: Ref<VerifyOptions>,
    source_provider: Ref<SourceProvider>,
    api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    targets: Ref<TargetFormatProvider>,
    paths: Ref<PathManager>,
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
            warn!("{} for transcoding {id}", "Unsuitable".bold());
            for issue in &result.issues {
                warn!("{issue}");
            }
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
        issues.append(&mut self.api_checks(source));
        issues.append(&mut self.flac_checks(source)?);
        if let Some(issue) = self.hash_check(source).await? {
            issues.push(issue);
        }
        Ok(VerifySuccess { issues })
    }

    /// Validate the source against the API.
    fn api_checks(&self, source: &Source) -> Vec<SourceIssue> {
        let exclude_tags = self.verify_options.exclude_tags.clone().unwrap_or_default();
        let target_formats = self.targets.get(source.format, &source.existing);
        let mut issues = Vec::new();
        issues.extend(check_category(source));
        issues.extend(check_scene(source));
        issues.extend(check_lossy_master(source));
        issues.extend(check_lossy_web(source));
        issues.extend(check_trumpable(source));
        issues.extend(check_unconfirmed(source));
        issues.extend(check_excluded_tags(source, &exclude_tags));
        issues.extend(check_existing_formats(source, &target_formats));
        issues
    }

    fn flac_checks(&self, source: &Source) -> Result<Vec<SourceIssue>, Failure<VerifyAction>> {
        if let Some(issue) = check_directory_exists(source) {
            return Ok(vec![issue]);
        }
        let flacs = Collector::get_flacs_with_context(&source.directory);
        if flacs.is_empty() {
            return Ok(vec![SourceIssue::NoFlacs {
                path: source.directory.clone(),
            }]);
        }
        let mut issues: Vec<SourceIssue> = Vec::new();
        issues.extend(check_flac_count(source, flacs.len()));

        issues.append(&mut VerifyCommand::subdirectory_checks(&flacs));

        let max_target = self
            .targets
            .get_max_path_length(source.format, &source.existing);
        let output_dir = self.paths.get_output_dir();
        for flac in flacs {
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
        TorrentVerifier::execute(&torrent_path, &source.directory)
            .await
            .map_err(Failure::wrap(VerifyAction::VerifyHash))
    }

    /// Retrieve the source `.torrent` file, downloading from the API if not cached.
    pub(crate) async fn get_source_torrent(
        &self,
        source: &Source,
    ) -> Result<PathBuf, Failure<VerifyAction>> {
        let path = self.paths.get_source_torrent_path(source);
        if path.is_file() {
            trace!("{} cached torrent file: {}", "Using".bold(), path.display());
            return Ok(path);
        }
        trace!(
            "{} torrent file as it's not cached: {}",
            "Downloading".bold(),
            path.display()
        );
        let torrents_dir = path.parent().expect("torrent path should have parent");
        if !torrents_dir.is_dir() {
            create_dir(torrents_dir).map_err(Failure::wrap_with_path(
                VerifyAction::CreateTorrentDirectory,
                torrents_dir,
            ))?;
        }
        let mut tmp_path = path.clone();
        tmp_path.as_mut_os_string().push(".tmp");
        let mut file = File::create(&tmp_path)
            .await
            .map_err(Failure::wrap_with_path(
                VerifyAction::CreateTorrentFile,
                &tmp_path,
            ))?;
        let buffer = self
            .api
            .download_torrent(source.torrent.id)
            .await
            .map_err(Failure::wrap(VerifyAction::DownloadTorrent))?;
        file.write_all(&buffer)
            .await
            .map_err(Failure::wrap_with_path(
                VerifyAction::WriteTorrentFile,
                &tmp_path,
            ))?;
        file.flush().await.map_err(Failure::wrap_with_path(
            VerifyAction::FlushTorrentFile,
            &tmp_path,
        ))?;
        drop(file);
        rename(&tmp_path, &path).await.map_err(Failure::wrap_with(
            VerifyAction::RenameTorrentFile,
            |f| {
                f.with("from", tmp_path.display().to_string())
                    .with("to", path.display().to_string())
            },
        ))?;
        Ok(path)
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
