use rogue_logging::Error;
use std::path::Path;
use tokio::process::Command;

use crate::dependencies::*;
use crate::utils::*;

pub struct MetaflacCommand;

impl MetaflacCommand {
    /// List tags and stream info for a flac file.
    async fn list(path: &Path) -> Result<String, Error> {
        let output = Command::new(METAFLAC)
            .arg("--list")
            .arg("--block-type=VORBIS_COMMENT")
            .arg("--block-type=STREAMINFO")
            .arg(path.to_string_lossy().to_string())
            .output()
            .await
            .map_err(|e| command_error(e, "get details", METAFLAC))?;
        let output = OutputHandler::execute(output, "get details", "metaflac")?;
        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    }

    /// List tags and stream info for a directory of flac files.
    pub async fn list_dir(path: &Path) -> Result<String, Error> {
        if !path.is_dir() {
            return Err(error("get details", "path is not a directory".to_owned()));
        }
        let mut output = String::new();
        let mut flacs = Collector::get_flacs(&path.to_path_buf());
        flacs.sort_by_key(|x| x.path.clone());
        for flac in flacs {
            let relative_path = flac.sub_dir.join(format!("{}.flac", flac.file_name));
            output.push_str("------------------------------\n");
            output.push_str(&relative_path.to_string_lossy());
            output.push_str("\n------------------------------\n");
            let details = Self::list(&flac.path).await?;
            output.push_str(&details);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rogue_logging::Error;
    use std::path::PathBuf;

    #[tokio::test]
    #[ignore = "sample required"]
    async fn metaflac_list() -> Result<(), Error> {
        // Arrange
        let paths = DirectoryReader::new()
            .with_extension("flac")
            .read(&PathBuf::from("./output"))
            .expect("Directory should exist");
        let path = paths.first().expect("Should be at least one sample");

        // Act
        println!("{path:?}");
        let output = MetaflacCommand::list(path).await?;
        println!("{output}");

        // Assert
        assert!(!output.is_empty());

        Ok(())
    }
}
