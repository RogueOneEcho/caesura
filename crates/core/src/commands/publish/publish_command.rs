use crate::prelude::*;
use gazelle_api::{GazelleClientTrait, UploadResponse};

/// Result of a publish operation.
#[derive(Debug)]
pub(crate) struct PublishSuccess {
    pub response: Option<UploadResponse>,
    pub permalink: Option<String>,
    pub next_transcode: Option<String>,
    pub next_upload: Option<String>,
}

/// Publish a source FLAC torrent from a local directory using a manifest.
#[injectable]
pub(crate) struct PublishCommand {
    arg: Ref<PublishArg>,
    shared_options: Ref<SharedOptions>,
    api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    paths: Ref<PathManager>,
}

impl PublishCommand {
    /// Execute `publish` from CLI.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<PublishAction>> {
        let indexer = self.shared_options.indexer_lowercase();
        if indexer != "red" {
            return Err(Failure::new(
                PublishAction::ValidateTracker,
                PublishError::UnsupportedIndexer { indexer },
            ));
        }

        let manifest = PublishManifest::read(&self.arg.publish_path)
            .map_err(Failure::wrap(PublishAction::ParseManifest))?;
        manifest
            .validate()
            .map_err(Failure::wrap(PublishAction::ValidateManifest))?;

        let result = self.execute(&manifest).await?;
        if let Some(response) = result.response {
            info!("{} source FLAC", "Published".bold());
            if let Some(link) = result.permalink {
                info!("{link}");
            }
            if let Some(next_transcode) = result.next_transcode {
                info!("Next: {next_transcode}");
            }
            if let Some(next_upload) = result.next_upload {
                info!("Next: {next_upload}");
            }
            trace!(
                "Published group_id={} torrent_id={}",
                response.group_id, response.torrent_id
            );
        } else {
            info!("{} source FLAC as this is a dry run", "Skipped".bold());
        }
        Ok(true)
    }

    /// Execute `publish` against an already-parsed manifest.
    #[expect(
        clippy::too_many_lines,
        reason = "publish orchestration intentionally keeps flow in one method"
    )]
    pub(crate) async fn execute(
        &self,
        manifest: &PublishManifest,
    ) -> Result<PublishSuccess, Failure<PublishAction>> {
        let indexer = self.shared_options.indexer_lowercase();
        if indexer != "red" {
            return Err(Failure::new(
                PublishAction::ValidateTracker,
                PublishError::UnsupportedIndexer { indexer },
            ));
        }

        let torrent_path = manifest.torrent_path.clone().unwrap_or_else(|| {
            let source_name = manifest
                .source_path
                .file_name()
                .expect("source_path should have a file name")
                .to_string_lossy()
                .to_string();
            self.paths.get_output_dir().join(format!(
                "{source_name}.{}.source.torrent",
                self.shared_options.indexer_lowercase()
            ))
        });

        TorrentCreator::create(
            &manifest.source_path,
            &torrent_path,
            self.shared_options.announce_url.clone(),
            self.shared_options.indexer.clone(),
        )
        .await
        .map_err(Failure::wrap(PublishAction::CreateTorrent))?;

        match manifest.mode {
            PublishMode::NewGroup => {
                let form = manifest.to_new_source_form(torrent_path);
                if manifest.dry_run {
                    info!("{} upload as this is a dry run", "Skipping".bold());
                    info!("{} data for source upload:", "Upload".bold());
                    info!("\n{form}");
                    return Ok(PublishSuccess {
                        response: None,
                        permalink: None,
                        next_transcode: None,
                        next_upload: None,
                    });
                }
                let response = self
                    .api
                    .upload_new_source(form)
                    .await
                    .map_err(Failure::wrap(PublishAction::UploadNewSource))?;
                let permalink = get_permalink(
                    &self.shared_options.indexer_url,
                    response.group_id,
                    response.torrent_id,
                );
                let next_transcode = format!("caesura transcode {}", response.torrent_id);
                let next_upload = format!("caesura upload {}", response.torrent_id);
                Ok(PublishSuccess {
                    response: Some(response),
                    permalink: Some(permalink),
                    next_transcode: Some(next_transcode),
                    next_upload: Some(next_upload),
                })
            }
            PublishMode::ExistingGroup => {
                let existing_group = manifest
                    .existing_group
                    .as_ref()
                    .expect("existing_group should be present after validation");
                let group = self
                    .api
                    .get_torrent_group(existing_group.group_id)
                    .await
                    .map_err(Failure::wrap(PublishAction::GetTorrentGroup))?;
                let probe_torrent = gazelle_api::Torrent {
                    media: existing_group.media.clone(),
                    format: existing_group.format.clone(),
                    encoding: existing_group.bitrate.clone(),
                    remaster_year: Some(existing_group.remaster_year),
                    remaster_title: existing_group.remaster_title.clone(),
                    remaster_record_label: existing_group.remaster_record_label.clone(),
                    remaster_catalogue_number: existing_group.remaster_catalogue_number.clone(),
                    ..gazelle_api::Torrent::default()
                };
                let existing = ExistingFormatProvider::get(&probe_torrent, &group.torrents);
                let source_format =
                    ExistingFormat::from_torrent(&probe_torrent).ok_or_else(|| {
                        Failure::new(
                            PublishAction::CheckDuplicate,
                            PublishError::UnsupportedSourceFormat,
                        )
                    })?;
                if existing.contains(&source_format) {
                    return Err(Failure::new(
                        PublishAction::CheckDuplicate,
                        PublishError::DuplicateSource,
                    ));
                }

                let form = manifest.to_existing_group_form(torrent_path);
                if manifest.dry_run {
                    info!("{} upload as this is a dry run", "Skipping".bold());
                    info!("{} data for source upload:", "Upload".bold());
                    info!("\n{form}");
                    return Ok(PublishSuccess {
                        response: None,
                        permalink: None,
                        next_transcode: None,
                        next_upload: None,
                    });
                }
                let response = self
                    .api
                    .upload_torrent(form)
                    .await
                    .map_err(Failure::wrap(PublishAction::UploadExistingGroup))?;
                let permalink = get_permalink(
                    &self.shared_options.indexer_url,
                    response.group_id,
                    response.torrent_id,
                );
                let next_transcode = format!("caesura transcode {}", response.torrent_id);
                let next_upload = format!("caesura upload {}", response.torrent_id);
                Ok(PublishSuccess {
                    response: Some(response),
                    permalink: Some(permalink),
                    next_transcode: Some(next_transcode),
                    next_upload: Some(next_upload),
                })
            }
        }
    }
}
