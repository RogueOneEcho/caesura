use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::*;

use crate::api::Api;
use crate::errors::AppError;
use crate::formats::TargetFormatProvider;
use crate::fs::{Collector, PathManager};
use crate::imdl::imdl_command::ImdlCommand;
use crate::naming::{Shortener, SourceName};
use crate::options::verify_options::VerifyOptions;
use crate::options::{Options, SharedOptions};
use crate::source::*;
use crate::verify::tag_verifier::TagVerifier;
use crate::verify::SourceRule::*;
use crate::verify::*;

/// Verify a FLAC source is suitable for transcoding.
#[injectable]
pub struct VerifyCommand {
    shared_options: Ref<SharedOptions>,
    verify_options: Ref<VerifyOptions>,
    source_provider: RefMut<SourceProvider>,
    api: RefMut<Api>,
    targets: Ref<TargetFormatProvider>,
    paths: Ref<PathManager>,
}

impl VerifyCommand {
    pub async fn execute(&mut self) -> Result<bool, AppError> {
        if !self.shared_options.validate() || !self.verify_options.validate() {
            return Ok(false);
        }
        let source = self
            .source_provider
            .write()
            .expect("Source provider should be writeable")
            .get_from_options()
            .await?;
        let errors = self.execute_internal(&source).await?;
        let is_verified = errors.is_empty();
        if is_verified {
            info!("{} {}", "Verified".bold(), source);
        } else {
            warn!("{} to verify {}", "Failed".bold(), source);
            for error in errors {
                warn!("{error}");
            }
        }
        Ok(is_verified)
    }

    pub async fn execute_internal(&mut self, source: &Source) -> Result<Vec<SourceRule>, AppError> {
        debug!("{} {}", "Verifying".bold(), source);
        Self::name_checks(source);
        let mut api_errors = self.api_checks(source);
        let mut flac_errors = self.flac_checks(source)?;
        let mut hash_check = if self
            .verify_options
            .no_hash_check
            .expect("no_hash_check should be set")
        {
            debug!("{} hash check due to settings", "Skipped".bold());
            Vec::new()
        } else {
            self.hash_check(source).await?
        };
        let mut errors: Vec<SourceRule> = Vec::new();
        errors.append(&mut api_errors);
        errors.append(&mut flac_errors);
        errors.append(&mut hash_check);
        Ok(errors)
    }

    fn name_checks(source: &Source) {
        let sanitized = SourceName::get(&source.metadata);
        let unsanitized = SourceName::get_unsanitized(&source.metadata);
        if sanitized != unsanitized {
            debug!("Source name has been sanitized: {sanitized}");
        }
    }

    fn api_checks(&self, source: &Source) -> Vec<SourceRule> {
        let mut errors: Vec<SourceRule> = Vec::new();
        if source.group.category_name != "Music" {
            errors.push(IncorrectCategory(source.group.category_name.clone()));
        }
        if source.torrent.scene {
            errors.push(SceneNotSupported);
        }
        if source.torrent.lossy_master_approved == Some(true) {
            errors.push(LossyMasterNeedsApproval);
        }
        if source.torrent.lossy_web_approved == Some(true) {
            errors.push(LossyWebNeedsApproval);
        }
        if source.torrent.trumpable == Some(true) {
            errors.push(TrumpableNotSuitable);
        }
        let target_formats = self.targets.get(source.format, &source.existing);
        if target_formats.is_empty() {
            errors.push(NoTranscodeFormats);
        }
        errors
    }

    fn flac_checks(&self, source: &Source) -> Result<Vec<SourceRule>, AppError> {
        if !source.directory.exists() || !source.directory.is_dir() {
            return Ok(vec![SourceDirectoryNotFound(
                source.directory.to_string_lossy().to_string(),
            )]);
        }
        let flacs = Collector::get_flacs(&source.directory);
        if flacs.is_empty() {
            return Ok(vec![NoFlacFiles(
                source.directory.to_string_lossy().to_string(),
            )]);
        }
        let mut errors: Vec<SourceRule> = Vec::new();
        for flac in flacs {
            let max_path = self.paths.get_max_transcode_sub_path(source, &flac);
            if max_path.len() > MAX_PATH_LENGTH {
                errors.push(PathTooLong(max_path));
                Shortener::suggest_track_name(&flac);
            }
            for error in TagVerifier::execute(&flac, source)? {
                errors.push(error);
            }
            for error in StreamVerifier::execute(&flac)? {
                errors.push(error);
            }
        }
        if errors.iter().any(|rule| matches!(rule, &PathTooLong(_))) {
            Shortener::suggest_album_name(source);
        }
        Ok(errors)
    }

    async fn hash_check(&mut self, source: &Source) -> Result<Vec<SourceRule>, AppError> {
        let mut api = self.api.write().expect("API should be available");
        let buffer = api.get_torrent_file_as_buffer(source.torrent.id).await?;
        ImdlCommand::verify_from_buffer(&buffer, &source.directory).await
    }
}
