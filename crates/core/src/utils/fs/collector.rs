use crate::prelude::*;

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

    /// Create [`FlacFile`] for each `.flac` file in a directory with [`DiscContext`] attached.
    ///
    /// This variant computes disc context (multi-disc detection, track padding) and
    /// attaches it to each flac. Use this when you need disc/track information for
    /// renaming or validation.
    #[must_use]
    pub fn get_flacs_with_context(source_dir: &PathBuf) -> Vec<FlacFile> {
        let mut flacs = Self::get_flacs(source_dir);
        let context = DiscContext::from_flacs(&flacs);
        for flac in &mut flacs {
            flac.disc_context = Some(context.clone());
        }
        flacs
    }

    /// Collect the FLAC files of a [`Source`], or the issue that blocks collection.
    ///
    /// - Returns [`SourceIssue::MissingDirectory`] when the directory is absent
    /// - Returns [`SourceIssue::NoFlacs`] when no FLAC files are present
    /// - Guards the directory check before collecting so [`Collector::get_flacs`] never panics
    pub(crate) fn collect_flacs(source: &Source) -> Result<Vec<FlacFile>, SourceIssue> {
        if !source.directory.is_dir() {
            return Err(SourceIssue::MissingDirectory {
                path: source.directory.clone(),
            });
        }
        trace!("Collecting FLACs from {}", source.directory.display());
        let flacs = Self::get_flacs_with_context(&source.directory);
        if flacs.is_empty() {
            return Err(SourceIssue::NoFlacs {
                path: source.directory.clone(),
            });
        }
        Ok(flacs)
    }

    /// Create [`AdditionalFile`] for each additional file in a directory.
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
