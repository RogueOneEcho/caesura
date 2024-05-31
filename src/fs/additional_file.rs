use std::path::PathBuf;

pub struct AdditionalFile {
    /// Path to the file
    pub path: PathBuf,

    /// File name with the extension.
    pub file_name: String,

    /// Subdirectory of the file.
    pub sub_dir: PathBuf,
}

impl AdditionalFile {
    #[must_use]
    pub fn new(path: PathBuf, source_dir: &PathBuf) -> Self {
        let sub_dir = path
            .strip_prefix(source_dir)
            .expect("Additional file path should start with the source directory")
            .parent()
            .expect("Additional file path should have a parent directory")
            .to_path_buf();
        let file_name = path
            .file_name()
            .expect("Additional file should have a name")
            .to_os_string()
            .to_string_lossy()
            .to_string();
        AdditionalFile {
            path,
            file_name,
            sub_dir,
        }
    }
}
