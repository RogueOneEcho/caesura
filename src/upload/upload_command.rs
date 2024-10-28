use std::path::{Path, PathBuf};

use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{error, info, trace, warn};
use tokio::fs::{copy, hard_link};

use crate::api::{Api, UploadForm};
use crate::built_info::*;
use crate::errors::AppError;
use crate::formats::{TargetFormat, TargetFormatProvider};
use crate::fs::{copy_dir, Collector, PathManager};
use crate::imdl::ImdlCommand;
use crate::jobs::Job;
use crate::options::{Options, SharedOptions, SourceArg, UploadOptions};
use crate::queue::TimeStamp;
use crate::source::{get_permalink, Source, SourceProvider};
use crate::transcode::{TranscodeJobFactory, Variant};
use crate::upload::UploadStatus;

const MUSIC_CATEGORY_ID: u8 = 0;

/// Upload transcodes of a FLAC source.
#[injectable]
pub struct UploadCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    upload_options: Ref<UploadOptions>,
    source_provider: RefMut<SourceProvider>,
    api: RefMut<Api>,
    paths: Ref<PathManager>,
    targets: Ref<TargetFormatProvider>,
    transcode_job_factory: Ref<TranscodeJobFactory>,
}

impl UploadCommand {
    /// Execute [`UploadCommand`] from the CLI.
    ///
    /// [`Source`] is retrieved from the CLI arguments.
    ///
    /// Returns `true` if all the uploads succeed.
    pub async fn execute_cli(&mut self) -> Result<bool, AppError> {
        if !self.arg.validate()
            || !self.shared_options.validate()
            || !self.upload_options.validate()
        {
            return Ok(false);
        }
        let source = self
            .source_provider
            .write()
            .expect("Source provider should be writeable")
            .get_from_options()
            .await
            .map_err(|e| AppError::else_explained("get source from options", e.to_string()))?;
        let status = self.execute(&source).await;
        // Errors were already printed as they occurred
        Ok(status.success)
    }

    /// Execute [`UploadCommand`] on a [`Source`].
    ///
    /// Returns an [`UploadStatus`] indicating the success of the operation and any errors.
    ///
    /// Errors are logged so do NOT need to be handled by the caller.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub async fn execute(&mut self, source: &Source) -> UploadStatus {
        let targets = self.targets.get(source.format, &source.existing);
        let mut api = self.api.write().expect("API should be available to read");
        let mut status = UploadStatus {
            success: true,
            formats: None,
            completed: TimeStamp::now(),
            errors: None,
        };
        let mut errors = Vec::new();
        for target in targets {
            let torrent_path = self.paths.get_torrent_path(source, target, true);
            if !torrent_path.exists() {
                warn!("In v0.19.0 the torrent file name format changed.");
                warn!("Running the transcode command will update existing transcodes without re-transcoding.");
                let error = AppError::else_explained(
                    "upload",
                    format!(
                        "The torrent file does not exist: {}",
                        torrent_path.display()
                    ),
                );
                error!("{error}");
                errors.push(error);
                status.success = false;
                continue;
            }
            let target_dir = self.paths.get_transcode_target_dir(source, target);
            trace!("{} content of {}", "Verifying".bold(), target_dir.display());
            if let Err(error) = ImdlCommand::verify(&torrent_path, &target_dir).await {
                let error =
                    AppError::else_external("verify torrent content", "IMDL", format!("{error}"));
                error!("{error}");
                errors.push(error);
                status.success = false;
                continue;
            }
            if let Some(torrent_dir) = &self.upload_options.copy_torrent_to {
                if let Err(error) = self.copy_torrent(source, &target, torrent_dir).await {
                    // If copy_torrent fails we can still continue with the upload
                    warn!("{error}");
                    errors.push(error);
                }
            }
            if self
                .upload_options
                .copy_transcode_to_content_dir
                .expect("copy_transcode_to_content_dir should be set")
            {
                trace!(
                    "{} {} to content directory",
                    "Copying".bold(),
                    target_dir.display()
                );
                if let Err(error) = self.copy_transcode(source, &target).await {
                    // If copy_transcode fails we can still continue with the upload
                    warn!("{error}");
                    errors.push(error);
                }
            }
            let form = UploadForm {
                path: torrent_path,
                category_id: MUSIC_CATEGORY_ID,
                remaster_year: source.metadata.year,
                remaster_title: source.torrent.remaster_title.clone(),
                remaster_record_label: source.torrent.remaster_record_label.clone(),
                remaster_catalogue_number: source.torrent.remaster_catalogue_number.clone(),
                format: target.get_file_extension().to_uppercase(),
                bitrate: target.get_bitrate().to_owned(),
                media: source.torrent.media.clone(),
                release_desc: self.create_description(source, target),
                group_id: source.group.id,
            };
            if self.upload_options.dry_run.expect("dry_run should be set") {
                warn!("{} upload as this is a dry run", "Skipping".bold());
                info!("{} data of {target} for {source}:", "Upload".bold());
                info!("{}", form);
                continue;
            }
            match api.upload_torrent(form).await {
                Ok(response) => {
                    info!("{} {target} for {source}", "Uploaded".bold());
                    let base = &self
                        .shared_options
                        .indexer_url
                        .clone()
                        .expect("indexer_url should be set");
                    let link =
                        get_permalink(base, response.get_group_id(), response.get_torrent_id());
                    info!("{link}");
                }
                Err(error) => {
                    error!("{error}");
                    errors.push(error);
                    status.success = false;
                    continue;
                }
            }
        }
        status.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };
        status
    }

    async fn copy_transcode(&self, source: &Source, target: &TargetFormat) -> Result<(), AppError> {
        let source_dir = self.paths.get_transcode_target_dir(source, *target);
        let source_dir_name = source_dir
            .file_name()
            .expect("source dir should have a name");
        let target_dir = self
            .shared_options
            .content
            .clone()
            .expect("content should be set")
            .first()
            .expect("content should contain at least one directory")
            .join(source_dir_name);
        let verb = if self
            .upload_options
            .hard_link
            .expect("hard_link should be set")
        {
            copy_dir(&source_dir, &target_dir, true).await?;
            "Hard Linked"
        } else {
            copy_dir(&source_dir, &target_dir, false).await?;
            "Copied"
        };
        trace!(
            "{} {} to {}",
            verb.bold(),
            source_dir.display(),
            target_dir.display()
        );
        Ok(())
    }

    async fn copy_torrent(
        &self,
        source: &Source,
        target: &TargetFormat,
        target_dir: &Path,
    ) -> Result<(), AppError> {
        let source_path = self.paths.get_torrent_path(source, *target, true);
        let source_file_name = source_path
            .file_name()
            .expect("torrent path should have a name");
        let target_path = target_dir.join(source_file_name);
        let verb = if self
            .upload_options
            .hard_link
            .expect("hard_link should be set")
        {
            hard_link(&source_path, &target_path)
                .await
                .or_else(|e| AppError::io(e, "hard link torrent file"))?;
            "Hard Linked"
        } else {
            copy(&source_path, &target_path)
                .await
                .or_else(|e| AppError::io(e, "copy torrent file"))?;
            "Copied"
        };
        trace!(
            "{} {} to {}",
            verb.bold(),
            source_path.display(),
            target_path.display()
        );
        Ok(())
    }

    #[allow(clippy::uninlined_format_args)]
    fn create_description(&self, source: &Source, target: TargetFormat) -> String {
        let base = &self
            .shared_options
            .indexer_url
            .clone()
            .expect("indexer_url should be set");
        let source_url = get_permalink(base, source.group.id, source.torrent.id);
        let source_title = source.format.get_title();
        let transcode_command = self.get_command(source, target).unwrap_or_else(|error| {
            warn!("Failed to get transcode command: {error}");
            String::new()
        });
        let lines: Vec<String> = [
            format!(
                "Transcoded and uploaded with [url={}][b]{}[/b] v{}[/url]",
                PKG_REPOSITORY, PKG_NAME, PKG_VERSION
            ),
            format!("[pad=0|10|0|20]Source[/pad] [url={source_url}]{source_title}[/url]"),
            format!("[pad=0|10|0|0]Command[/pad] [code]{transcode_command}[/code]"),
            format!(
                "[url={}]Learn how easy it is to create and upload transcodes yourself![/url]",
                PKG_REPOSITORY
            ),
        ]
        .iter()
        .map(|line| format!("[quote]{line}[/quote]"))
        .collect();

        lines.join("")
    }

    pub fn get_command(&self, source: &Source, target: TargetFormat) -> Result<String, AppError> {
        let flacs = Collector::get_flacs(&source.directory);
        let flac = flacs.first().expect("Should be at least one FLAC");
        let job = self
            .transcode_job_factory
            .create_single(0, flac, source, target)?;
        let Job::Transcode(job) = job else {
            return AppError::explained(
                "get transcode command",
                "expected a transcode job".to_owned(),
            );
        };
        let command = match job.variant {
            Variant::Transcode(mut decode, mut encode) => {
                decode.input = PathBuf::from("input.flac");
                let extension = encode
                    .output
                    .extension()
                    .expect("output should have an extension")
                    .to_string_lossy();
                encode.output = PathBuf::from(format!("output.{extension}"));
                format!(
                    "{} | {}",
                    decode.to_info().display(),
                    encode.to_info().display()
                )
            }
            Variant::Resample(mut resample) => {
                resample.input = PathBuf::from("input.flac");
                let extension = resample
                    .output
                    .extension()
                    .expect("output should have an extension")
                    .to_string_lossy();
                resample.output = PathBuf::from(format!("output.{extension}"));
                resample.to_info().display()
            }
        };
        Ok(command)
    }
}
