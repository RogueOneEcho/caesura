use di::{injectable, Ref};

use crate::commands::*;
use crate::utils::*;

/// A factory for creating [`SpectrogramJob`] from multiple flac files.
#[injectable]
pub(crate) struct SpectrogramJobFactory {
    paths: Ref<PathManager>,
}

impl SpectrogramJobFactory {
    /// Create a [`SpectrogramJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
    #[must_use]
    pub(crate) fn create(&self, flacs: &[FlacFile], source: &Source) -> Vec<Job> {
        let mut jobs = Vec::new();
        for (index, flac) in flacs.iter().enumerate() {
            jobs.push(self.create_single(source, index, flac, Size::Zoom));
            jobs.push(self.create_single(source, index, flac, Size::Full));
        }
        jobs
    }

    /// Create a single [`SpectrogramJob`] instance from `flac_file`.
    ///
    /// Arguments:
    ///
    /// * `flac_file`: Path to the flac file.
    /// * `size`: Size of the spectrogram to create.
    fn create_single(&self, source: &Source, index: usize, flac: &FlacFile, size: Size) -> Job {
        let out_filename = flac.file_name.clone()
            + match size {
                Size::Full => ".full.png",
                Size::Zoom => ".zoom.png",
            };
        let id = format!("Spectrogram {size:<4?}{index:>3}");
        let source_path = flac.get_path_string();
        let output_path = self
            .paths
            .get_spectrogram_dir(source)
            .join(&flac.sub_dir)
            .join(out_filename);
        let image_title = flac.file_name.clone();
        Job::Spectrogram(SpectrogramJob {
            id,
            source_path,
            output_path,
            image_title,
            size,
        })
    }
}
