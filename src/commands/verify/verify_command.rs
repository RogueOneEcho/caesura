use colored::Colorize;
use di::{Ref, RefMut, injectable};
use log::*;

use crate::commands::*;
use crate::dependencies::*;
use crate::options::*;
use crate::utils::*;

use crate::utils::SourceIssue::*;
use gazelle_api::GazelleClient;
use rogue_logging::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub(crate) struct VerifyCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    verify_options: Ref<VerifyOptions>,
    source_provider: RefMut<SourceProvider>,
    api: RefMut<GazelleClient>,
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
    pub(crate) async fn execute_cli(&mut self) -> Result<bool, Error> {
        if !self.arg.validate()
            || !self.shared_options.validate()
            || !self.verify_options.validate()
        {
            return Ok(false);
        }
        let source = self
            .source_provider
            .write()
            .expect("Source provider should be writeable")
            .get_from_options()
            .await;
        let (status, id) = match source {
            Ok(source) => (self.execute(&source).await, source.to_string()),
            Err(issue) => (VerifyStatus::from_issue(issue), "unknown".to_owned()),
        };
        if status.verified {
            info!("{} {id}", "Verified".bold());
        } else {
            warn!("{} for transcoding {id}", "Unsuitable".bold());
            if let Some(issues) = &status.issues {
                for issue in issues {
                    warn!("{issue}");
                }
            }
        }
        Ok(status.verified)
    }

    /// Execute [`VerifyCommand`] on a [`Source`].
    ///
    /// [`SourceIssue`] issues are not logged so must be handled by the caller.
    #[must_use]
    pub(crate) async fn execute(&mut self, source: &Source) -> VerifyStatus {
        debug!("{} {}", "Verifying".bold(), source);
        let mut issues: Vec<SourceIssue> = Vec::new();
        issues.append(&mut self.api_checks(source));
        issues.append(&mut self.flac_checks(source));
        issues.append(&mut self.hash_check(source).await);
        VerifyStatus::from_issues(issues)
    }

    /// Validate the source against the API.
    fn api_checks(&self, source: &Source) -> Vec<SourceIssue> {
        let mut issues: Vec<SourceIssue> = Vec::new();
        if source.group.category_name != "Music" {
            issues.push(Category {
                actual: source.group.category_name.clone(),
            });
        }
        if source.torrent.scene {
            issues.push(Scene);
        }
        if source.torrent.lossy_master_approved == Some(true) {
            issues.push(LossyMaster);
        }
        if source.torrent.lossy_web_approved == Some(true) {
            issues.push(LossyWeb);
        }
        if source.torrent.trumpable == Some(true) {
            issues.push(Trumpable);
        }
        if !source.torrent.remastered {
            issues.push(Unconfirmed);
        }
        let excluded_tags: Vec<String> = self
            .verify_options
            .exclude_tags
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|x| source.group.tags.contains(x))
            .collect();
        if !excluded_tags.is_empty() {
            issues.push(Excluded {
                tags: excluded_tags,
            });
        }
        let target_formats = self.targets.get(source.format, &source.existing);
        if target_formats.is_empty() {
            issues.push(Existing {
                formats: source.existing.clone(),
            });
        }
        issues
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        clippy::as_conversions
    )]
    fn flac_checks(&self, source: &Source) -> Vec<SourceIssue> {
        if !source.directory.is_dir() {
            return vec![MissingDirectory {
                path: source.directory.clone(),
            }];
        }
        let flacs = Collector::get_flacs(&source.directory);
        if flacs.is_empty() {
            return vec![NoFlacs {
                path: source.directory.clone(),
            }];
        }
        let mut issues: Vec<SourceIssue> = Vec::new();
        let api_flacs = source.torrent.get_flacs();
        if flacs.len() != api_flacs.len() {
            issues.push(FlacCount {
                expected: api_flacs.len(),
                actual: flacs.len(),
            });
        }

        issues.append(&mut VerifyCommand::subdirectory_checks(&flacs));

        let max_target = self
            .targets
            .get_max_path_length(source.format, &source.existing);
        let output_dir = self.paths.get_output_dir();
        let mut too_long = false;
        for flac in flacs {
            if let Some(max_target) = max_target {
                let path = self
                    .paths
                    .get_transcode_path(source, max_target, &flac)
                    .strip_prefix(output_dir.clone())
                    .expect("should be able to strip prefix from transcode path")
                    .to_path_buf();
                let length = path.to_string_lossy().chars().count() as isize;
                let excess = length - MAX_PATH_LENGTH;
                if excess > 0 {
                    let excess = excess as usize;
                    issues.push(Length { path, excess });
                    Shortener::suggest_track_name(&flac);
                    too_long = true;
                }
            }
            let tags = TagVerifier::execute(&flac, source)
                .unwrap_or(vec!["failed to retrieve tags".to_owned()]);
            if !tags.is_empty() {
                issues.push(MissingTags {
                    path: flac.path.clone(),
                    tags,
                });
            }
            for error in StreamVerifier::execute(&flac) {
                issues.push(error);
            }
        }
        if too_long {
            Shortener::suggest_album_name(source);
        }
        issues
    }

    async fn hash_check(&mut self, source: &Source) -> Vec<SourceIssue> {
        if self
            .verify_options
            .no_hash_check
            .expect("no_hash_check should be set")
        {
            debug!("{} hash check due to settings", "Skipped".bold());
            return Vec::new();
        }
        let mut api = self.api.write().expect("API should be available");
        let torrent_path = self.paths.get_source_torrent_path(source);
        if !torrent_path.is_file() {
            trace!(
                "{} torrent file as it's not cached: {}",
                "Downloading".bold(),
                torrent_path.display()
            );
            let mut file = match File::create_new(&torrent_path).await {
                Ok(file) => file,
                Err(e) => {
                    return vec![SourceIssue::Error {
                        domain: "File System".to_owned(),
                        details: e.to_string(),
                    }];
                }
            };
            let buffer = match api.download_torrent(source.torrent.id).await {
                Ok(buffer) => buffer,
                Err(e) => return vec![SourceIssue::api(e)],
            };
            if let Err(e) = file.write_all(&buffer).await {
                return vec![SourceIssue::Error {
                    domain: "File System".to_owned(),
                    details: e.to_string(),
                }];
            }
            if let Err(e) = file.flush().await {
                return vec![SourceIssue::Error {
                    domain: "File System".to_owned(),
                    details: e.to_string(),
                }];
            }
        }
        ImdlCommand::verify(&torrent_path, &source.directory)
            .await
            .unwrap_or_else(|e| {
                Some(SourceIssue::Error {
                    domain: "IMDL".to_owned(),
                    details: e.to_string(),
                })
            })
            .map_or_else(Vec::new, |x| vec![x])
    }

    pub fn subdirectory_checks(flacs: &[FlacFile]) -> Vec<SourceIssue> {
        // source.directory is the root directory of the torrent. If all flacs share a subdirectory
        // within that, it is unnecessary and trumpable. Multi-disc sets may separate items by
        // subdirs, so they will not be a common prefix.
        // Note that this is meant to verify the most common case, where a single unnecessary
        // directory contains all flac content, likely due to a misunderstanding of how the
        // creation tool works.
        let flac_sub_dirs = flacs.iter().map(|x| x.sub_dir.clone()).collect::<Vec<_>>();
        if let Some(prefix) = Shortener::longest_common_prefix(&flac_sub_dirs) {
            return vec![UnnecessaryDirectory { prefix }];
        }
        vec![]
    }
}
