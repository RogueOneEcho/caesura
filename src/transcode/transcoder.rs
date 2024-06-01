use std::path::Path;

use colored::Colorize;
use di::{injectable, Ref};
use log::*;

use crate::formats::{TargetFormat, TargetFormatProvider};
use crate::fs::Collector;
use crate::jobs::{AppError, JobRunner};
use crate::logging::Colors;
use crate::naming::{DirectoryName, SourceName};
use crate::options::SharedOptions;
use crate::source::*;
use crate::transcode::{AdditionalJobFactory, TranscodeJobFactory};

const OUTPUT_SUB_DIR: &str = "transcodes";

/// Transcode a [Source].
#[injectable]
pub struct SourceTranscoder {
    shared_options: Ref<SharedOptions>,
    targets: Ref<TargetFormatProvider>,
    transcode_job_factory: Ref<TranscodeJobFactory>,
    additional_job_factory: Ref<AdditionalJobFactory>,
    runner: Ref<JobRunner>,
}

impl SourceTranscoder {
    pub async fn execute(&self, source: &Source) -> Result<bool, AppError> {
        let targets = self.targets.get(source.format, &source.existing);
        let dir_name = SourceName::get_escaped(source);
        let output_dir = &self
            .shared_options
            .output
            .clone()
            .expect("Option should be set")
            .join(dir_name)
            .join(OUTPUT_SUB_DIR);
        self.execute_transcode(source, &targets, output_dir).await?;
        self.execute_additional(source, &targets, output_dir)
            .await?;
        debug!(
            "{} {}",
            "in".gray(),
            output_dir.to_string_lossy().to_string().gray()
        );
        Ok(true)
    }

    pub async fn execute_transcode(
        &self,
        source: &Source,
        targets: &Vec<TargetFormat>,
        output_dir: &Path,
    ) -> Result<(), AppError> {
        let flacs = Collector::get_flacs(&source.directory);
        info!(
            "{} to {:?} for {} FLACs in {}",
            "Transcoding".bold(),
            targets,
            flacs.len().to_string().gray(),
            source
        );
        for target in targets {
            let dir_name = DirectoryName::get(source, target);
            let output_dir = output_dir.join(dir_name);
            let jobs = self
                .transcode_job_factory
                .create(&flacs, *target, &output_dir)?;
            self.runner.add(jobs);
        }
        self.runner.execute().await?;
        info!("{} {}", "Transcoded".bold(), source);
        Ok(())
    }

    pub async fn execute_additional(
        &self,
        source: &Source,
        targets: &Vec<TargetFormat>,
        output_dir: &Path,
    ) -> Result<(), AppError> {
        let files = Collector::get_additional(&source.directory);
        info!(
            "{} {} additional files",
            "Adding".bold(),
            files.len().to_string().gray()
        );
        for target in targets {
            let dir_name = DirectoryName::get(source, target);
            let output_dir = output_dir.join(dir_name);
            let jobs = self
                .additional_job_factory
                .create(&files, *target, &output_dir);
            self.runner.add(jobs);
        }
        self.runner.execute().await?;
        info!("{} additional files {}", "Added".bold(), source);
        debug!(
            "{} {}",
            "in".gray(),
            output_dir.to_string_lossy().to_string().gray()
        );
        Ok(())
    }
}
