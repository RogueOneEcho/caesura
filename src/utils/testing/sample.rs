use crate::utils::*;
use colored::Colorize;
use log::trace;
use rogue_logging::Error;
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Path to the CURL binary.
#[cfg(target_os = "windows")]
const CURL: &str = "curl.exe";

/// Path to the CURL binary.
#[cfg(not(target_os = "windows"))]
const CURL: &str = "curl";

/// Path to the SHA256SUM binary.
#[cfg(target_os = "windows")]
const SHA256SUM: &str = "sha256sum.exe";

/// Path to the SHA256SUM binary.
#[cfg(not(target_os = "windows"))]
const SHA256SUM: &str = "sha256sum";

/// Path to the UNZIP binary.
#[cfg(target_os = "windows")]
const UNZIP: &str = "unzip.exe";

/// Path to the SHA256SUM binary.
#[cfg(not(target_os = "windows"))]
const UNZIP: &str = "unzip";

pub(crate) struct Sample {
    pub name: String,
    pub url: String,
    pub dir_name: String,
    pub sha256: String,
}

impl Sample {
    pub(crate) fn get_content_path(&self) -> PathBuf {
        PathBuf::from("samples/content").join(&self.dir_name)
    }

    pub(crate) fn exists(&self) -> bool {
        let content_path = self.get_content_path();
        content_path.is_dir()
    }

    pub(crate) async fn fetch(&self) -> Result<(), Error> {
        let zip_path = temp_dir().join(format!("{}.zip", self.sha256));
        trace!("{} sample {}", "Downloading".bold(), self.name);
        download(&self.url, &zip_path).await?;
        trace!("{} sample {}", "Validating".bold(), self.name);
        validate(&zip_path, &self.sha256).await?;
        let content_path = self.get_content_path();
        trace!("{} sample {}", "Extracting".bold(), self.name);
        unzip(&zip_path, &content_path).await?;
        Ok(())
    }
}

async fn download(url: &str, path: &Path) -> Result<(), Error> {
    let output = Command::new(CURL)
        .arg("--location")
        .arg("--create-dirs")
        .arg("--output")
        .arg(path.to_string_lossy().to_string())
        .arg(url)
        .output()
        .await
        .map_err(|e| command_error(e, "download sample", CURL))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(output_error(output, "download sample", CURL))
    }
}

async fn validate(path: &Path, expected: &str) -> Result<(), Error> {
    let output = Command::new(SHA256SUM)
        .arg(path.to_string_lossy().to_string())
        .output()
        .await
        .map_err(|e| command_error(e, "validate sample", SHA256SUM))?;
    if !output.status.success() {
        return Err(output_error(output, "validate sample", SHA256SUM));
    }
    let out = String::from_utf8(output.stdout).unwrap_or_default();
    let actual: String = out.chars().take(64).collect();
    if actual == expected {
        Ok(())
    } else {
        Err(error(
            "validate sample",
            format!("Expected: {expected}\nActual: {actual}"),
        ))
    }
}

async fn unzip(source: &Path, target: &Path) -> Result<(), Error> {
    let output = Command::new(UNZIP)
        .arg(source.to_string_lossy().to_string())
        .arg("-d")
        .arg(target)
        .output()
        .await
        .map_err(|e| command_error(e, "extract sample", UNZIP))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(output_error(output, "extract sample", UNZIP))
    }
}
