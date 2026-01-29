use super::FileSnapshot;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

/// Builder for creating a snapshot of files in a directory.
#[derive(Default)]
pub struct DirectorySnapshot {
    directory: Option<PathBuf>,
    excluded_extensions: Vec<String>,
}

impl DirectorySnapshot {
    /// Create a new [`DirectorySnapshot`] builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the directory to snapshot.
    #[must_use]
    pub fn with_directory(mut self, path: impl AsRef<Path>) -> Self {
        self.directory = Some(path.as_ref().to_path_buf());
        self
    }

    /// Exclude files with these extensions (case-insensitive).
    #[must_use]
    pub fn without_extensions(mut self, extensions: &[&str]) -> Self {
        self.excluded_extensions = extensions.iter().map(|s| (*s).to_owned()).collect();
        self
    }

    /// Create the snapshot, returning a sorted list of file snapshots.
    ///
    /// # Panics
    ///
    /// Panics if `with_directory` was not called.
    pub fn create(self) -> Result<Vec<FileSnapshot>> {
        let root = self
            .directory
            .expect("with_directory must be called before create");
        let mut files = Vec::new();
        collect_files(&root, &root, &mut files, &self.excluded_extensions)?;
        files.sort();
        Ok(files)
    }
}

fn collect_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<FileSnapshot>,
    exclude: &[String],
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_files(root, &path, files, exclude)?;
            continue;
        }

        if is_excluded(&path, exclude) {
            continue;
        }

        files.push(FileSnapshot::from_path(root, &path)?);
    }
    Ok(())
}

fn is_excluded(path: &Path, exclude: &[String]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| exclude.iter().any(|ex| ex.eq_ignore_ascii_case(ext)))
}
