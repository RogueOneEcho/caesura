use crate::prelude::*;
use std::os::unix::prelude::MetadataExt;

/// A non-FLAC file to include in transcodes.
pub struct AdditionalFile {
    /// Path to the existing file on disk
    ///
    /// - May contain invalid UTF-8 characters or decomposed (NFD) sequences
    pub path: PathBuf,

    /// Output file name with extension
    ///
    /// - Invalid UTF-8 is replaced with `�`
    /// - Decomposed (NFD) sequences are replaced with composed (NFC)
    pub file_name: String,

    /// Output subdirectory, relative to the source directory
    ///
    /// - Invalid UTF-8 is replaced with `�`
    /// - Decomposed (NFD) sequences are replaced with composed (NFC)
    pub sub_dir: PathBuf,
}

impl AdditionalFile {
    /// Create a new [`AdditionalFile`] from a path.
    #[must_use]
    pub fn new(path: PathBuf, source_dir: &PathBuf) -> Self {
        let sub_dir = path
            .strip_prefix(source_dir)
            .expect("Additional file path should start with the source directory")
            .parent()
            .expect("Additional file path should have a parent directory")
            .to_nfc();
        let file_name = path
            .file_name()
            .expect("Additional file should have a name")
            .to_string_lossy()
            .to_nfc();
        AdditionalFile {
            path,
            file_name,
            sub_dir,
        }
    }

    /// File size in bytes.
    pub async fn get_size(&self) -> Result<u64, Failure<FsAction>> {
        let file = TokioFile::open(&self.path)
            .await
            .map_err(Failure::wrap_with_path(FsAction::OpenFile, &self.path))?;
        let metadata = file
            .metadata()
            .await
            .map_err(Failure::wrap_with_path(FsAction::ReadMetadata, &self.path))?;
        Ok(metadata.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decomposed sequences in the file name and subdirectory are composed.
    #[test]
    fn additional_file_new_decomposed() {
        // Arrange
        let source_dir = PathBuf::from("/music");
        let path = PathBuf::from("/music/No\u{308}rd/cover a\u{301}.jpg");
        // Act
        let file = AdditionalFile::new(path, &source_dir);
        // Assert
        assert_eq!(file.file_name, "cover \u{e1}.jpg");
        assert_eq!(file.sub_dir, PathBuf::from("N\u{f6}rd"));
    }
}
