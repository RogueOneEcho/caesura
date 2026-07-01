use crate::prelude::*;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub(crate) struct VerifyCommand {
    source_provider: Ref<SourceProvider>,
    api_verifier: Ref<ApiVerifier>,
    content_verifier: Ref<ContentVerifier>,
    flac_verifier: Ref<FlacVerifier>,
    decode_verifier: Ref<DecodeVerifier>,
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
        if let Some(issue) = self.content_verifier.execute(source).await? {
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
}
