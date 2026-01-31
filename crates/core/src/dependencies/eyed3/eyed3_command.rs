use crate::prelude::*;
use tokio::process::Command;

/// Facade for the `eyeD3` CLI binary.
///
/// Invokes `eyeD3` as a subprocess for MP3 tag inspection.
pub struct EyeD3Command;

impl EyeD3Command {
    /// Display tags and metadata for an audio file.
    pub async fn display(path: &Path) -> Result<String, Error> {
        let output = Command::new(EYED3)
            .arg(path.to_string_lossy().to_string())
            .arg("--no-color")
            .arg("-r")
            .run()
            .await
            .map_err(|e| process_error(e, "get details", EYED3))?;
        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use rogue_logging::Error;

    #[tokio::test]
    #[cfg_attr(target_arch = "aarch64", ignore = "Transcode output differs on ARM")]
    async fn eyed3_display() -> Result<(), Error> {
        // Arrange
        let transcode_config =
            TranscodeProvider::get(SampleFormat::default(), TargetFormat::_320).await;
        let mut paths = DirectoryReader::new()
            .with_extension("mp3")
            .read(&transcode_config.transcode_dir())
            .expect("Directory should exist");
        paths.sort();
        let path = paths.first().expect("Should be at least one sample");

        // Act
        println!("{path:?}");
        let output = EyeD3Command::display(path).await?;
        println!("{output}");

        // Assert
        assert_snapshot!(output);

        Ok(())
    }
}
