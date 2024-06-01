use std::fs::create_dir_all;
use std::path::Path;

use colored::Colorize;
use di::{injectable, Ref};
use log::*;

use crate::errors::AppError;
use crate::formats::{TargetFormat, TargetFormatProvider};
use crate::fs::Collector;
use crate::imdl::ImdlCommand;
use crate::jobs::JobRunner;
use crate::logging::Colors;
use crate::naming::{SourceName, TargetName};
use crate::options::SharedOptions;
use crate::source::*;
use crate::transcode::{AdditionalJobFactory, TranscodeJobFactory};

const TRANSCODE_SUB_DIR: &str = "transcodes";
const TORRENT_SUB_DIR: &str = "torrents";

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
            .join(TRANSCODE_SUB_DIR);
        self.execute_transcode(source, &targets, output_dir).await?;
        self.execute_additional(source, &targets, output_dir)
            .await?;
        self.execute_torrent(source, &targets).await?;
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
            let dir_name = TargetName::get(source, target);
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
            let dir_name = TargetName::get(source, target);
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

    pub async fn execute_torrent(
        &self,
        source: &Source,
        targets: &Vec<TargetFormat>,
    ) -> Result<(), AppError> {
        info!("{} torrents {}", "Creating".bold(), source);
        let dir_name = SourceName::get_escaped(source);
        let dir = &self
            .shared_options
            .output
            .clone()
            .expect("Option should be set")
            .join(dir_name);
        let torrent_dir = dir.join(TORRENT_SUB_DIR);
        create_dir_all(&torrent_dir).or_else(|e| AppError::io(e, "create torrent directory"))?;
        for target in targets {
            let name = TargetName::get(source, target);
            let content_dir = dir.join(TRANSCODE_SUB_DIR).join(&name);
            let output_path = torrent_dir.join(name + ".torrent");
            let announce_url = self.get_announce_url();
            let indexer = self
                .shared_options
                .indexer
                .clone()
                .expect("option should be set");
            ImdlCommand::create(&content_dir, &output_path, announce_url, indexer).await?;
            debug!("{} torrent {:?}", "Created".bold(), output_path);
        }
        info!("{} torrents {}", "Created".bold(), source);
        Ok(())
    }

    fn get_announce_url(&self) -> String {
        let tracker_url = self
            .shared_options
            .tracker_url
            .clone()
            .expect("option should be set");
        let api_key = self
            .shared_options
            .api_key
            .clone()
            .expect("option should be set");
        format!("{tracker_url}/{api_key}/announce")
    }
}
