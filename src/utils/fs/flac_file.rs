use std::path::PathBuf;

use claxon::metadata::StreamInfo;
use claxon::FlacReader;

/// A representation of a FLAC file.
pub struct FlacFile {
    /// Path to the file
    pub path: PathBuf,

    /// File name without the extension.
    pub file_name: String,

    /// Subdirectory of the file.
    pub sub_dir: PathBuf,
}

impl FlacFile {
    #[must_use]
    pub fn new(path: PathBuf, source_dir: &PathBuf) -> Self {
        let sub_dir = path
            .strip_prefix(source_dir)
            .expect("Flac file path should start with the source directory")
            .parent()
            .expect("Flac file path should have a parent directory")
            .to_path_buf();
        let file_name = path
            .file_name()
            .expect("Flac file should have a name")
            .to_os_string()
            .to_string_lossy()
            .strip_suffix(".flac")
            .expect("Flac file should .flac extension")
            .to_owned();
        FlacFile {
            path,
            file_name,
            sub_dir,
        }
    }

    #[must_use]
    pub fn get_path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn get_stream_info(&self) -> Result<StreamInfo, claxon::Error> {
        let reader = FlacReader::open(&self.path)?;
        Ok(reader.streaminfo())
    }
}
