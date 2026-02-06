use crate::prelude::*;
use tokio::process::Command;

/// Facade for the `metaflac` CLI binary.
///
/// Invokes `metaflac` as a subprocess for FLAC metadata inspection.
pub struct MetaflacCommand;

impl MetaflacCommand {
    /// List tags and stream info for a flac file.
    async fn list(path: &Path) -> Result<String, Failure<MetaflacAction>> {
        let output = Command::new(METAFLAC)
            .arg("--list")
            .arg("--block-type=VORBIS_COMMENT")
            .arg("--block-type=STREAMINFO")
            .arg(path.to_string_lossy().to_string())
            .run()
            .await
            .map_err(Failure::wrap_with_path(MetaflacAction::GetDetails, path))?;
        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    }

    /// List tags and stream info for a directory of flac files.
    pub async fn list_dir(path: &Path) -> Result<String, Failure<MetaflacAction>> {
        if !path.is_dir() {
            return Err(Failure::new(
                MetaflacAction::GetDetails,
                IoError::new(ErrorKind::NotADirectory, "path is not a directory"),
            )
            .with_path(path));
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
    use crate::utils::SAMPLE_SOURCES_DIR;
    use insta::assert_snapshot;

    #[tokio::test]
    async fn metaflac_list() -> Result<(), Failure<MetaflacAction>> {
        // Arrange
        let album = AlbumProvider::get(SampleFormat::default()).await;
        let mut paths = DirectoryReader::new()
            .with_extension("flac")
            .read(&SAMPLE_SOURCES_DIR.join(album.dir_name()))
            .expect("Directory should exist");
        paths.sort();
        let path = paths.first().expect("Should be at least one sample");

        // Act
        println!("{path:?}");
        let output = MetaflacCommand::list(path).await?;
        println!("{output}");

        // Assert
        assert_snapshot!(output);

        Ok(())
    }
}
