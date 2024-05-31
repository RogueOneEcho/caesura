use std::path::Path;

use di::{injectable, Ref};

use crate::formats::target_format::TargetFormat;
use crate::fs::AdditionalFile;
use crate::jobs::Job;
use crate::options::TranscodeOptions;
use crate::transcode::AdditionalJob;

#[injectable]
pub struct AdditionalJobFactory {
    options: Ref<TranscodeOptions>,
}

impl AdditionalJobFactory {
    /// Create a [`AdditionalJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
    #[must_use]
    pub fn create(
        &self,
        files: &[AdditionalFile],
        format: TargetFormat,
        output_dir: &Path,
    ) -> Vec<Job> {
        let mut jobs = Vec::new();
        for (index, file) in files.iter().enumerate() {
            jobs.push(self.create_single(index, file, format, output_dir));
        }
        jobs
    }

    /// Create a single [`AdditionalJob`] from a `flac_file`.
    fn create_single(
        &self,
        index: usize,
        file: &AdditionalFile,
        format: TargetFormat,
        output_dir: &Path,
    ) -> Job {
        let id = format!("Additional {format:<7?}{index:>3}");
        let source_path = file.path.clone();
        let output_dir = output_dir.join(&file.sub_dir);
        let output_path = output_dir
            .join(&file.file_name)
            .to_string_lossy()
            .into_owned();
        let output_dir = output_dir.to_string_lossy().into_owned();
        let extension = source_path
            .extension()
            .expect("Source has extension")
            .to_string_lossy()
            .into_owned();
        let compress_images = self.options.compress_images.expect("Options should be set");
        let hard_link = self.options.hard_link.expect("Options should be set");
        Job::Additional(AdditionalJob {
            id,
            source_path,
            output_dir,
            output_path,
            hard_link,
            compress_images,
            extension,
        })
    }
}
