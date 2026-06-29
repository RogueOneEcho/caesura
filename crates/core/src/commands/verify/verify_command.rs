use crate::prelude::*;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub(crate) struct VerifyCommand {
    verify_options: Ref<VerifyOptions>,
    source_provider: Ref<SourceProvider>,
    api_verifier: Ref<ApiVerifier>,
    flac_verifier: Ref<FlacVerifier>,
    decode_verifier: Ref<DecodeVerifier>,
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
        if !issues.is_empty() {
            trace!("Skipping hash and FLAC checks as API checks failed");
            return Ok(VerifySuccess { issues });
        }
        if let Some(issue) = self.hash_check(source).await? {
            issues.push(issue);
            trace!("Skipping FLAC checks as hash check failed");
            return Ok(VerifySuccess { issues });
        }
        match Collector::collect_flacs(source) {
            Ok(flacs) => {
                issues.append(&mut self.flac_verifier.execute(source, &flacs)?);
                if issues.is_empty() {
                    issues.append(&mut self.decode_verifier.execute(&flacs).await);
                } else {
                    trace!("Skipping decode check as FLAC checks failed");
                }
            }
            Err(issue) => issues.push(issue),
        }
        if let Err(failure) = self.reporter.execute(source, &issues) {
            warn!("{}", failure.render());
        }
        Ok(VerifySuccess { issues })
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
}
