use colored::Colorize;
use di::{injectable, Ref, RefMut};
use log::*;

use crate::api::Api;
use crate::formats::TargetFormatProvider;
use crate::fs::{Collector, FlacFile};
use crate::imdl::imdl_command::ImdlCommand;
use crate::imdl::ImdlError::IOFailure;
use crate::source::SourceError::*;
use crate::source::*;
use crate::verify::SourceRule::*;
use crate::verify::*;

/// Check if a [Source] is suitable for transcoding.
#[injectable]
pub struct SourceVerifier {
    api: RefMut<Api>,
    targets: Ref<TargetFormatProvider>,
}

impl SourceVerifier {
    pub async fn execute(&mut self, source: &Source) -> Result<bool, SourceError> {
        info!("{} {}", "Verifying".bold(), source);
        let api_errors = self.api_checks(source);
        debug_errors(&api_errors, source, "API checks");
        let flac_errors = self.flac_checks(source)?;
        debug_errors(&flac_errors, source, "FLAC file checks");
        let hash_check = self.hash_check(source).await?;
        debug_errors(&hash_check, source, "Hash check");
        let is_verified = api_errors.is_empty() && flac_errors.is_empty() && hash_check.is_empty();
        if is_verified {
            info!("{} {}", "Verified".bold(), source);
        } else {
            warn!("{} {}", "Skipped".bold().yellow(), source);
            warn_errors(api_errors);
            warn_errors(flac_errors);
            warn_errors(hash_check);
        }
        Ok(is_verified)
    }

    fn api_checks(&self, source: &Source) -> Vec<SourceRule> {
        let mut errors: Vec<SourceRule> = Vec::new();
        if source.torrent.scene {
            errors.push(SceneNotSupported);
        }
        if source.torrent.lossy_master_approved == Some(true) {
            errors.push(LossyMasterNeedsApproval);
        }
        if source.torrent.lossy_web_approved == Some(true) {
            errors.push(LossyWebNeedsApproval);
        }
        let target_formats = self.targets.get(source.format, &source.existing);
        if target_formats.is_empty() {
            errors.push(NoTranscodeFormats);
        }
        errors
    }

    fn flac_checks(&self, source: &Source) -> Result<Vec<SourceRule>, SourceError> {
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
            let mut tags = match validate_vinyl_tags(&flac, &source.metadata.media) {
                Ok(tags) => tags,
                Err(error) => return Err(AudioTagFailure(error)),
            };
            errors.append(&mut tags);
        }
        Ok(errors)
    }

    async fn hash_check(&mut self, source: &Source) -> Result<Vec<SourceRule>, SourceError> {
        let mut api = self.api.write().expect("API should be available");
        let buffer = match api.get_torrent_file_as_buffer(source.torrent.id).await {
            Ok(buffer) => buffer,
            Err(error) => return Err(ApiFailure(error)),
        };
        let is_verified = match ImdlCommand::verify(&buffer, &source.directory).await {
            Ok(is_verified) => is_verified,
            Err(error) => return Err(ImdlFailure(IOFailure(error))),
        };
        if is_verified {
            Ok(vec![])
        } else {
            Ok(vec![IncorrectHash])
        }
    }
}
fn validate_vinyl_tags(
    flac: &FlacFile,
    media: &String,
) -> Result<Vec<SourceRule>, audiotags::Error> {
    // TODO MUST confirm vinyl media comparison works
    if media != "Vinyl" {
        validate_tags_internal(flac)
    } else {
        let errors = validate_tags_internal(flac)?;
        let count_before = errors.len();
        let errors: Vec<SourceRule> = errors
            .into_iter()
            // TODO MUST confirm NoTrackNumberTag error filter works
            .filter(|error| !matches!(*error, NoTrackNumberTag(_)))
            .collect();
        if count_before != errors.len() {
            warn!("Unable to verify if the track number is valid. Vinyl releases can have non-standard track numbers (e.g. A1, A2, etc).");
        }
        Ok(errors)
    }
}

fn validate_tags_internal(flac: &FlacFile) -> Result<Vec<SourceRule>, audiotags::Error> {
    let tags = flac.get_tags()?;
    let mut errors: Vec<SourceRule> = Vec::new();
    if tags.artist().is_none() {
        errors.push(NoArtistTag(flac.file_name.clone()));
    }
    if tags.album().is_none() {
        errors.push(NoAlbumTag(flac.file_name.clone()));
    }
    if tags.title().is_none() {
        errors.push(NoTitleTag(flac.file_name.clone()));
    }
    if tags.track_number().is_none() {
        errors.push(NoTrackNumberTag(flac.file_name.clone()));
    }
    Ok(errors)
}

fn debug_errors(errors: &Vec<SourceRule>, source: &Source, title: &str) {
    if errors.is_empty() {
        debug!("{} {} {}", "Passed".bold(), title, source);
    } else {
        debug!("{} {} {}", "Failed".bold().red(), title, source);
        for error in errors {
            debug!("{} {}", "⚠".yellow(), error);
        }
    }
}

fn warn_errors(errors: Vec<SourceRule>) {
    for error in errors {
        warn!("{}", error);
    }
}
