use crate::prelude::*;

/// Regex capturing the semantic version from `flac --version` output.
pub(crate) const FLAC_VERSION_PATTERN: &str = r"flac (\d+\.\d+(?:\.\d+)?)";
/// Regex capturing the semantic version from `lame --version` output.
pub(crate) const LAME_VERSION_PATTERN: &str = r"version (\d+\.\d+)";
/// Regex capturing the semantic version from `sox --version` output.
pub(crate) const SOX_VERSION_PATTERN: &str = r"v(\d+\.\d+\.\d+(?:\.\d+)?)";

/// Display version information for caesura and its dependencies.
#[injectable]
pub struct VersionCommand {
    sox: Ref<SoxFactory>,
    release_provider: Ref<ReleaseProvider>,
}

impl VersionCommand {
    /// Execute the version command.
    ///
    /// Returns `true` if all dependencies are found, `false` if any are missing.
    pub async fn execute(&self) -> bool {
        let versions = self.get_versions().await;
        let found = versions.check_versions();
        if !found {
            error!("Failed to find all dependencies\n");
        }
        let table = build_table(versions);
        print!("{table}");
        match self.release_provider.get_latest().await {
            Ok(release) => release.log(),
            Err(failure) => warn!("{}", failure.render()),
        }
        found
    }

    /// Get the version of each dependency.
    pub(super) async fn get_versions(&self) -> DependencyVersions<'_> {
        let sox_binary = self.sox.binary();
        let flac = get_version(FLAC, FLAC_VERSION_PATTERN).await;
        let lame = get_version(LAME, LAME_VERSION_PATTERN).await;
        let sox = get_version(sox_binary, SOX_VERSION_PATTERN).await;
        DependencyVersions([(FLAC, flac), (LAME, lame), (sox_binary, sox)])
    }
}

/// Check if a dependency is available and extract its version.
pub(super) async fn get_version(binary: &str, pattern: &str) -> Result<VersionInfo, VersionError> {
    let output = TokioCommand::new(binary)
        .arg("--version")
        .output()
        .await
        .map_err(VersionError::Process)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout
        .lines()
        .next()
        .ok_or(VersionError::EmptyStdout)?
        .to_owned();
    let version = Regex::new(pattern)
        .expect("version pattern should be valid")
        .captures(&first_line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_owned());
    Ok(VersionInfo {
        first_line,
        version,
    })
}

/// Build the version table.
fn build_table(versions: DependencyVersions<'_>) -> String {
    let mut builder = TableBuilder::new().row([
        APP_NAME.to_owned(),
        app_version_or_describe().trim_start_matches('v').to_owned(),
        app_user_agent(false).dimmed().to_string(),
    ]);
    for (name, result) in versions.0 {
        let (version, detail) = match result {
            Ok(info) => (
                info.version.unwrap_or_else(|| String::from("?")),
                info.first_line.dimmed().to_string(),
            ),
            Err(e) => ("⚠".yellow().to_string(), e.to_string().yellow().to_string()),
        };
        builder = builder.row([name.to_owned(), version, detail]);
    }
    builder.build()
}

/// Version information for a dependency.
pub(super) struct VersionInfo {
    /// First line of version output.
    pub(super) first_line: String,
    /// Extracted version number, if regex matched.
    pub(super) version: Option<String>,
}

impl VersionInfo {
    /// Create a [`VersionInfo`] with mock values for testing.
    #[cfg(test)]
    pub(super) fn mock() -> Self {
        Self {
            first_line: "example 1.0".to_owned(),
            version: Some("1.0".to_owned()),
        }
    }
}

/// Version information for each dependency, keyed by binary name.
pub(super) struct DependencyVersions<'a>(
    pub(super) [(&'a str, Result<VersionInfo, VersionError>); 3],
);

impl DependencyVersions<'_> {
    /// Were all dependencies found?
    pub(super) fn check_versions(&self) -> bool {
        self.0.iter().all(|(_, result)| result.is_ok())
    }
}

/// Errors returned by [`get_version`].
#[derive(Debug, ThisError)]
pub(super) enum VersionError {
    #[error("Unable to get version information")]
    EmptyStdout,
    #[error("Unable to run dependency: {0}")]
    Process(IoError),
}
