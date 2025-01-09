use crate::commands::*;
use crate::utils::*;

use colored::Colorize;
use log::trace;
use std::path::PathBuf;

/// A collection of [`FlacFile`].
pub struct Collector;

impl Collector {
    /// Create [`FlacFile`] for each `.flac` file in a directory.
    #[must_use]
    pub fn get_flacs(source_dir: &PathBuf) -> Vec<FlacFile> {
        let paths = DirectoryReader::new()
            .with_extension("flac")
            .read(source_dir)
            .expect("Source directory should be readable");
        let mut collection = Vec::new();
        for path in paths {
            collection.push(FlacFile::new(path, source_dir));
        }
        trace!(
            "{} {} flacs in: {}",
            "Found".bold(),
            collection.len(),
            source_dir.display()
        );
        collection
    }

    /// Create [`AdditionalFile`] for each additonal file in a directory.
    #[must_use]
    pub fn get_additional(source_dir: &PathBuf) -> Vec<AdditionalFile> {
        let paths = DirectoryReader::new()
            .with_max_depth(1)
            .with_extensions(IMAGE_EXTENSIONS.to_vec())
            .with_extensions(TEXT_EXTENSIONS.to_vec())
            .read(source_dir)
            .expect("Source directory should be readable");
        let mut collection = Vec::new();
        for path in paths {
            collection.push(AdditionalFile::new(path, source_dir));
        }
        trace!(
            "{} {} files in: {}",
            "Found".bold(),
            collection.len(),
            source_dir.display()
        );
        collection
    }
}
