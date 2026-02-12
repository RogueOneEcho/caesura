use crate::built_info::{PKG_NAME, PKG_VERSION};
use crate::prelude::*;
use regex::Regex;
use tokio::process::Command;

pub(crate) const FLAC_VERSION_PATTERN: &str = r"flac (\d+\.\d+(?:\.\d+)?)";
pub(crate) const LAME_VERSION_PATTERN: &str = r"version (\d+\.\d+)";
pub(crate) const SOX_VERSION_PATTERN: &str = r"v(\d+\.\d+\.\d+)";

/// Display version information for caesura and its dependencies.
///
/// Note: sox on macOS outputs `sox: SoX v` without the version number
/// so the sox version will display as "?".
/// - <https://sourceforge.net/p/sox/patches/104/>
#[injectable]
pub struct VersionCommand;

impl VersionCommand {
    /// Execute the version command.
    ///
    /// Returns `true` if all dependencies are found, `false` if any are missing.
    pub async fn execute(&self) -> bool {
        let flac = get_version(FLAC, FLAC_VERSION_PATTERN).await;
        let lame = get_version(LAME, LAME_VERSION_PATTERN).await;
        let sox = get_version(SOX, SOX_VERSION_PATTERN).await;
        let dependencies = [(FLAC, flac), (LAME, lame), (SOX, sox)];
        let any_error = dependencies.iter().any(|(_, result)| result.is_err());
        if any_error {
            error!("Failed to find all dependencies");
            for (name, result) in &dependencies {
                if let Err(err) = result {
                    eprintln!("    {name}: {err}");
                }
            }
        }
        let table = build_table(dependencies);
        print!("{table}");
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
    let mut builder =
        TableBuilder::new().row([PKG_NAME.to_owned(), PKG_VERSION.to_owned(), String::new()]);
    for (name, result) in dependencies {
        let (version, detail) = match result {
            Ok(info) => (
                info.version.unwrap_or_else(|| String::from("?")),
                info.first_line.dimmed().to_string(),
            ),
            Err(e) => (String::from("?"), e.to_string().red().to_string()),
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
