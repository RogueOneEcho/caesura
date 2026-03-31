use crate::prelude::*;
use regex::Regex;
use tokio::process::Command;

pub(crate) const FLAC_VERSION_PATTERN: &str = r"flac (\d+\.\d+(?:\.\d+)?)";
pub(crate) const LAME_VERSION_PATTERN: &str = r"version (\d+\.\d+)";
pub(crate) const SOX_VERSION_PATTERN: &str = r"v(\d+\.\d+\.\d+(?:\.\d+)?)";

/// Display version information for caesura and its dependencies.
#[injectable]
pub struct VersionCommand {
    sox: Ref<SoxFactory>,
}

impl VersionCommand {
    /// Execute the version command.
    ///
    /// Returns `true` if all dependencies are found, `false` if any are missing.
    pub async fn execute(&self) -> bool {
        let sox_binary = self.sox.binary();
        let flac = get_version(FLAC, FLAC_VERSION_PATTERN).await;
        let lame = get_version(LAME, LAME_VERSION_PATTERN).await;
        let sox = get_version(sox_binary, SOX_VERSION_PATTERN).await;
        let dependencies = [(FLAC, flac), (LAME, lame), (sox_binary, sox)];
        let any_error = dependencies.iter().any(|(_, result)| result.is_err());
        if any_error {
            error!("Failed to find all dependencies\n");
        }
        let table = build_table(dependencies);
        print!("{table}");
        GitHubRelease::check_for_update().await;
        !any_error
    }
}

/// Check if a dependency is available and extract its version.
pub(super) async fn get_version(binary: &str, pattern: &str) -> Result<VersionInfo, VersionError> {
    let output = Command::new(binary)
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
fn build_table(dependencies: [(&str, Result<VersionInfo, VersionError>); 3]) -> String {
    let mut builder = TableBuilder::new().row([
        APP_NAME.to_owned(),
        app_version_or_describe().trim_start_matches('v').to_owned(),
        app_user_agent(false).dimmed().to_string(),
    ]);
    for (name, result) in dependencies {
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

/// Errors returned by [`get_version`].
#[derive(Debug, ThisError)]
pub(super) enum VersionError {
    #[error("Unable to get version information")]
    EmptyStdout,
    #[error("Unable to run dependency: {0}")]
    Process(IoError),
}
