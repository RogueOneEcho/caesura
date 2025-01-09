use std::io::Error;
use std::path::{Path, PathBuf};

pub struct DirectoryReader {
    included_extensions: Vec<String>,
    max_depth: Option<usize>,
}

impl DirectoryReader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            included_extensions: Vec::new(),
            max_depth: None,
        }
    }

    pub fn with_extension(&mut self, extension: &str) -> &mut Self {
        self.included_extensions.push(extension.to_owned());
        self
    }

    pub fn with_extensions(&mut self, extensions: Vec<&str>) -> &mut Self {
        for extension in extensions {
            self.included_extensions.push(extension.to_owned());
        }
        self
    }

    pub fn with_max_depth(&mut self, max_depth: usize) -> &mut Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn read(&mut self, path: &Path) -> Result<Vec<PathBuf>, Error> {
        let depth = 0;
        let mut files: Vec<PathBuf> = Vec::new();
        self.read_internal(path, depth, &mut files)?;
        Ok(files)
    }

    fn read_internal(
        &mut self,
        path: &Path,
        depth: usize,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), Error> {
        let dir = path.read_dir()?;
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if self.is_depth_ok(depth) {
                    self.read_internal(&path, depth + 1, files)?;
                }
            } else {
                if !self.is_extension_included(&path) {
                    continue;
                }
                files.push(path);
            }
        }
        Ok(())
    }

    fn is_extension_included(&self, path: &Path) -> bool {
        if self.included_extensions.is_empty() {
            true
        } else if let Some(extension) = path.extension() {
            let extension = extension.to_string_lossy().to_string();
            self.included_extensions.contains(&extension)
        } else {
            false
        }
    }

    fn is_depth_ok(&self, depth: usize) -> bool {
        if let Some(max_depth) = self.max_depth {
            depth < max_depth
        } else {
            true
        }
    }
}
