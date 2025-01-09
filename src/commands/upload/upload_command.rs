use std::collections::HashSet;
use std::ops::Not;
use std::path::{Path, PathBuf};

use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::{info, trace, warn};
use tokio::fs::{copy, hard_link};

use crate::built_info::{PKG_NAME, PKG_REPOSITORY, PKG_VERSION};
use crate::commands::*;
use crate::dependencies::*;
use crate::options::*;
use crate::utils::*;
use gazelle_api::{GazelleClient, UploadForm};
use rogue_logging::Error;
use TargetFormat::*;

const MUSIC_CATEGORY_ID: u8 = 0;

/// Upload transcodes of a FLAC source.
#[injectable]
pub(crate) struct UploadCommand {
    arg: Ref<SourceArg>,
    shared_options: Ref<SharedOptions>,
    upload_options: Ref<UploadOptions>,
    copy_options: Ref<CopyOptions>,
    source_provider: RefMut<SourceProvider>,
    api: RefMut<GazelleClient>,
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
    pub(crate) async fn execute_cli(&mut self) -> Result<bool, Error> {
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
            .map_err(|e| error("get source from options", e.to_string()))?;
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
    pub(crate) async fn execute(&mut self, source: &Source) -> UploadStatus {
        let targets = self.targets.get(source.format, &source.existing);
        let mut api = self.api.write().expect("API should be available to read");
        let mut status = UploadStatus {
            success: true,
            formats: None,
            completed: TimeStamp::now(),
            errors: None,
        };
        let mut errors = Vec::new();
        let mut formats = Vec::new();
        for target in targets {
            let torrent_path = self.paths.get_torrent_path(source, target, true);
            if !torrent_path.exists() {
                warn!("In v0.19.0 the torrent file name format changed.");
                warn!("Running the transcode command will update existing transcodes without re-transcoding.");
                let error = error(
                    "upload",
                    format!(
                        "The torrent file does not exist: {}",
                        torrent_path.display()
                    ),
                );
                error.log();
                errors.push(error);
                status.success = false;
                continue;
            }
            let target_dir = self.paths.get_transcode_target_dir(source, target);
            trace!("{} content of {}", "Verifying".bold(), target_dir.display());
            if let Err(e) = ImdlCommand::verify(&torrent_path, &target_dir).await {
                let error = error("verify torrent content", e.to_string());
                error.log();
                error.log();
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
                trace!("{} transcode to content directory", "Copying".bold());
                let destination = self
                    .shared_options
                    .content
                    .clone()
                    .expect("content should be set");
                let destination = destination
                    .first()
                    .expect("content should contain at least one directory");
                if let Err(error) = self.copy_transcode(&target_dir, destination).await {
                    // If copy_transcode fails we can still continue with the upload
                    warn!("{error}");
                    errors.push(error);
                }
            }
            if let Some(destination) = &self.upload_options.copy_transcode_to {
                trace!(
                    "{} transcode to: {}",
                    "Copying".bold(),
                    destination.display(),
                );
                if let Err(error) = self.copy_transcode(&target_dir, destination).await {
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
                release_desc: self.create_description(source, target).await,
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
                    let id = response.get_torrent_id();
                    let link = get_permalink(base, response.get_group_id(), id);
                    info!("{link}");
                    formats.push(UploadFormatStatus { format: target, id });
                }
                Err(error) => {
                    error.log();
                    errors.push(error);
                    status.success = false;
                    continue;
                }
            }
        }
        status.errors = errors.is_empty().not().then_some(errors);
        status.formats = formats.is_empty().not().then_some(formats);
        status
    }

    async fn copy_transcode(&self, source_path: &Path, target_parent: &Path) -> Result<(), Error> {
        let source_dir_name = source_path
            .file_name()
            .expect("source dir should have a name");
        let target_dir = target_parent.join(source_dir_name);
        if target_dir.exists() {
            warn!(
                "{} copy as the target directory already exists: {}",
                "Skipping".bold(),
                target_dir.display()
            );
            return Ok(());
        }
        let verb = if self
            .copy_options
            .hard_link
            .expect("hard_link should be set")
        {
            copy_dir(source_path, &target_dir, true).await?;
            "Hard Linked"
        } else {
            copy_dir(source_path, &target_dir, false).await?;
            "Copied"
        };
        trace!(
            "{} {} to {}",
            verb.bold(),
            source_path.display(),
            target_dir.display()
        );
        Ok(())
    }

    async fn copy_torrent(
        &self,
        source: &Source,
        target: &TargetFormat,
        target_dir: &Path,
    ) -> Result<(), Error> {
        let source_path = self.paths.get_torrent_path(source, *target, true);
        let source_file_name = source_path
            .file_name()
            .expect("torrent path should have a name");
        let target_path = target_dir.join(source_file_name);
        let verb = if self
            .copy_options
            .hard_link
            .expect("hard_link should be set")
        {
            hard_link(&source_path, &target_path)
                .await
                .map_err(|e| io_error(e, "hard link torrent file"))?;
            "Hard Linked"
        } else {
            copy(&source_path, &target_path)
                .await
                .map_err(|e| io_error(e, "copy torrent file"))?;
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
    async fn create_description(&self, source: &Source, target: TargetFormat) -> String {
        let base = &self
            .shared_options
            .indexer_url
            .clone()
            .expect("indexer_url should be set");
        let source_url = get_permalink(base, source.group.id, source.torrent.id);
        let source_title = source.format.get_title();
        let mut lines: Vec<String> = vec![
            format!(
                "Transcoded and uploaded with [url={}][b]{}[/b] v{}[/url]",
                PKG_REPOSITORY, PKG_NAME, PKG_VERSION
            ),
            format!("[pad=0|10|0|20]Source[/pad] [url={source_url}]{source_title}[/url]"),
        ];
        for transcode_command in self.get_commands(source, target) {
            lines.push(format!(
                "[pad=0|10|0|0]Transcode[/pad] [code]{transcode_command}[/code]"
            ));
        }
        match self.get_details(source, target).await {
            Ok(details) => {
                lines.push(format!(
                    "[pad=0|10|0|19]Details[/pad] [hide][pre]{details}[/pre][/hide]"
                ));
            }
            Err(error) => warn!("Failed to get transcode details: {error}"),
        }
        lines.push(format!(
            "[url={}]Learn how easy it is to create and upload transcodes yourself![/url]",
            PKG_REPOSITORY
        ));
        lines.into_iter().fold(String::new(), |mut output, line| {
            output.push_str("[quote]");
            output.push_str(&line);
            output.push_str("[/quote]");
            output
        })
    }

    pub(crate) fn get_commands(&self, source: &Source, target: TargetFormat) -> HashSet<String> {
        let flacs = Collector::get_flacs(&source.directory);
        flacs
            .into_iter()
            .filter_map(|flac| {
                self.get_command_internal(flac, source, target)
                    .unwrap_or_else(|e| {
                        warn!("{e}");
                        None
                    })
            })
            .collect()
    }

    fn get_command_internal(
        &self,
        flac: FlacFile,
        source: &Source,
        target: TargetFormat,
    ) -> Result<Option<String>, Error> {
        let job = self
            .transcode_job_factory
            .create_single(0, &flac, source, target)?;
        let Job::Transcode(job) = job else {
            return Err(error(
                "get transcode command",
                "expected a transcode job".to_owned(),
            ));
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
                Some(format!(
                    "{} | {}",
                    decode.to_info().display(),
                    encode.to_info().display()
                ))
            }
            Variant::Resample(mut resample) => {
                resample.input = PathBuf::from("input.flac");
                let extension = resample
                    .output
                    .extension()
                    .expect("output should have an extension")
                    .to_string_lossy();
                resample.output = PathBuf::from(format!("output.{extension}"));
                Some(resample.to_info().display())
            }
            Variant::Include(_) => None,
        };
        Ok(command)
    }

    async fn get_details(&self, source: &Source, target: TargetFormat) -> Result<String, Error> {
        let path = self.paths.get_transcode_target_dir(source, target);
        match target {
            Flac => MetaflacCommand::list_dir(&path).await,
            _320 | V0 => EyeD3Command::display(&path).await,
        }
    }
}
