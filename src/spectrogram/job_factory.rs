use std::path::Path;

use di::injectable;

use crate::fs::FlacFile;
use crate::jobs::Job;
use crate::spectrogram::*;

/// A factory for creating [`SpectrogramJob`] from multiple flac files.
#[injectable]
pub struct SpectrogramJobFactory;

impl SpectrogramJobFactory {
    /// Create a [`SpectrogramJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
    #[must_use]
    pub fn create(&self, flacs: &[FlacFile], output_dir: &Path) -> Vec<Job> {
        let mut jobs = Vec::new();
        for (index, flac) in flacs.iter().enumerate() {
            jobs.push(self.create_single(index, flac, Size::Zoom, output_dir));
            jobs.push(self.create_single(index, flac, Size::Full, output_dir));
        }
        jobs
    }

    /// Create a single [`SpectrogramJob`] instance from `flac_file`.
    ///
    /// Arguments:
    ///
    /// * `flac_file`: Path to the flac file.
    /// * `size`: Size of the spectrogram to create.
    fn create_single(&self, index: usize, flac: &FlacFile, size: Size, output_dir: &Path) -> Job {
        let out_filename = flac.file_name.clone()
            + match size {
                Size::Full => ".full.png",
                Size::Zoom => ".zoom.png",
            };
        let id = format!("Spectrogram {size:<4?}{index:>3}");
        let source_path = flac.get_path_string();
        let output_dir = output_dir.join(&flac.sub_dir);
        let output_path = output_dir.join(out_filename).to_string_lossy().into_owned();
        let output_dir = output_dir.to_string_lossy().into_owned();
        let image_title = flac.file_name.clone();
        Job::Spectrogram(SpectrogramJob {
            id,
            source_path,
            output_dir,
            output_path,
            image_title,
            size,
        })
    }
}
