use colored::Colorize;
use di::{injectable, Ref};
use log::{debug, info};

use crate::errors::AppError;
use crate::fs::*;
use crate::jobs::JobRunner;
use crate::logging::Colors;
use crate::naming::SourceName;
use crate::options::SharedOptions;
use crate::source::Source;
use crate::spectrogram::*;

const OUTPUT_SUB_DIR: &str = "spectrograms";

/// Generate spectrogram images from flac files.
#[injectable]
pub struct SpectrogramGenerator {
    options: Ref<SharedOptions>,
    factory: Ref<SpectrogramJobFactory>,
    runner: Ref<JobRunner>,
}

impl SpectrogramGenerator {
    /// Generate spectrogram images from flac files.
    pub async fn execute(&self, source: &Source) -> Result<bool, AppError> {
        info!("{} spectrograms for {}", "Creating".bold(), source);
        let collection = Collector::get_flacs(&source.directory);
        let dir_name = SourceName::get_escaped(source);
        let output_dir = self
            .options
            .output
            .clone()
            .expect("Option should be set")
            .join(dir_name)
            .join(OUTPUT_SUB_DIR);
        let jobs = self.factory.create(&collection, &output_dir);
        let count = jobs.len();
        self.runner.add(jobs);
        self.runner.execute().await?;
        info!("{} {} spectrograms for {}", "Created".bold(), count, source);
        debug!(
            "{} {}",
            "in".gray(),
            output_dir.to_string_lossy().to_string().gray()
        );
        Ok(true)
    }
}
