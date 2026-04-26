use crate::prelude::*;

/// Upload transcodes of a FLAC source.
#[injectable]
pub(crate) struct UploadCommand {
    shared_options: Ref<SharedOptions>,
    upload_options: Ref<UploadOptions>,
    copy_options: Ref<CopyOptions>,
    source_provider: Ref<SourceProvider>,
    api: Ref<GazelleClient>,
    paths: Ref<PathManager>,
    transcode_job_factory: Ref<TranscodeJobFactory>,
    qbit_options: Ref<QbitOptions>,
    qbit_upload_options: Ref<QbitUploadOptions>,
    injector: Ref<TorrentInjector>,
}

impl UploadCommand {
    /// Execute [`UploadCommand`] from the CLI.
    ///
    /// [`Source`] is retrieved from the CLI arguments.
    ///
    /// Returns `true` if all the uploads succeed.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<UploadAction>> {
        let source = self
            .source_provider
            .get_from_options()
            .await
            .map_err(Failure::wrap(UploadAction::GetSource))?
            .map_err(Failure::wrap(UploadAction::GetSource))?;
        self.execute(&source).await?;
        Ok(true)
    }

    /// Execute [`UploadCommand`] on a [`Source`].
    ///
    /// Returns an [`UploadSuccess`] on success, or a [`Failure`] on error.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn execute(
        &self,
        source: &Source,
    ) -> Result<UploadSuccess, Failure<UploadAction>> {
        if self.qbit_upload_options.inject_torrent {
            let mut errors: Vec<OptionRule> = Vec::new();
            self.qbit_options.validate_connection(&mut errors);
            if !errors.is_empty() {
                OptionRule::show(&errors);
                return Err(Failure::from_action(UploadAction::InjectTorrent));
            }
        }
        let mut warnings = Vec::new();
        let mut formats = Vec::new();
        for &target in &source.targets {
            let torrent_path = self.paths.get_torrent_path(source, target);
            if !torrent_path.exists() {
                warn!("In v0.19.0 the torrent file name format changed.");
                warn!(
                    "Running the transcode command will update existing transcodes without re-transcoding."
                );
                return Err(
                    Failure::new(UploadAction::FindTorrent, UploadError::MissingTorrent)
                        .with_path(&torrent_path),
                );
            }
            let target_dir = self.paths.get_transcode_target_dir(source, target);
            trace!("{} content of {}", "Verifying".bold(), target_dir.display());
            TorrentVerifier::execute(&torrent_path, &target_dir)
                .await
                .map_err(Failure::wrap(UploadAction::VerifyContent))?;
            let form = UploadForm {
                path: torrent_path.clone(),
                category_id: Category::Music,
                remaster_year: source.metadata.year,
                remaster_title: source.torrent.remaster_title.clone(),
                remaster_record_label: source.torrent.remaster_record_label.clone(),
                remaster_catalogue_number: source.torrent.remaster_catalogue_number.clone(),
                format: target.to_format(),
                bitrate: target.to_quality(),
                media: source.torrent.media.clone(),
                release_desc: self.create_description(source, target),
                group_id: source.group.id,
            };
            if self.upload_options.dry_run {
                warn!("{} upload as this is a dry run", "Skipping".bold());
                info!("{} data of {target} for {source}:", "Upload".bold());
                info!("\n{form}");
                continue;
            }
            if self.upload_options.copy_transcode_to_content_dir {
                trace!("{} transcode to content directory", "Copying".bold());
                let destination = self
                    .shared_options
                    .content
                    .first()
                    .expect("content should contain at least one directory");
                if let Err(e) = self.copy_transcode(&target_dir, destination).await {
                    warn!("{}", e.render());
                    warnings.push(e.to_error());
                }
            }
            if let Some(destination) = &self.upload_options.copy_transcode_to {
                trace!(
                    "{} transcode to: {}",
                    "Copying".bold(),
                    destination.display(),
                );
                if let Err(e) = self.copy_transcode(&target_dir, destination).await {
                    warn!("{}", e.render());
                    warnings.push(e.to_error());
                }
            }
            if let Some(torrent_dir) = &self.upload_options.copy_torrent_to {
                let source_torrent = self.paths.get_torrent_path(source, target);
                if let Err(e) = self
                    .injector
                    .copy_torrent(&source_torrent, torrent_dir)
                    .await
                    .map_err(Failure::wrap(UploadAction::CopyTorrent))
                {
                    warn!("{}", e.render());
                    warnings.push(e.to_error());
                }
            }
            let response = self
                .api
                .upload_torrent(form)
                .await
                .map_err(Failure::wrap(UploadAction::Upload))?;
            trace!("{} {target} for {source}", "Uploaded".bold());
            trace!(
                "{}",
                get_permalink(
                    &self.shared_options.indexer_url,
                    response.group_id,
                    response.torrent_id,
                )
            );
            if self.qbit_upload_options.inject_torrent {
                let add_options = self.qbit_upload_options.to_add_torrent_options();
                if let Err(e) = self
                    .injector
                    .inject_qbit(&torrent_path, add_options)
                    .await
                    .map_err(Failure::wrap(UploadAction::InjectTorrent))
                {
                    warn!("{}", e.render());
                    warnings.push(e.to_error());
                }
            }
            formats.push(UploadFormatStatus {
                format: target,
                id: response.torrent_id,
            });
        }
        if !formats.is_empty() {
            let base = &self.shared_options.indexer_url;
            let targets: Vec<_> = formats
                .iter()
                .map(|u| {
                    let link = get_torrent_permalink(base, u.id);
                    u.format.to_string().gray().italic().hyperlink(&link)
                })
                .collect();
            info!(
                "{} {} for {source}",
                "Uploaded".bold(),
                join_humanized(targets)
            );
        }
        Ok(UploadSuccess { formats, warnings })
    }

    async fn copy_transcode(
        &self,
        source_path: &Path,
        target_parent: &Path,
    ) -> Result<(), Failure<UploadAction>> {
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
        let verb = if self.copy_options.hard_link {
            copy_dir(source_path, &target_dir, true)
                .await
                .map_err(Failure::wrap(UploadAction::CopyTranscode))?;
            "Hard Linked"
        } else {
            copy_dir(source_path, &target_dir, false)
                .await
                .map_err(Failure::wrap(UploadAction::CopyTranscode))?;
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

    #[allow(clippy::uninlined_format_args)]
    fn create_description(&self, source: &Source, target: TargetFormat) -> String {
        let base = &self.shared_options.indexer_url;
        let source_url = get_permalink(base, source.group.id, source.torrent.id);
        let source_title = source.format.get_title();
        let mut lines: Vec<String> = vec![
            format!(
                "Transcoded and uploaded with [url={}][b]{}[/b] {}[/url]",
                APP_HOMEPAGE,
                APP_NAME,
                app_version_or_describe()
            ),
            format!("[pad=0|10|0|20]Source[/pad] [url={source_url}]{source_title}[/url]"),
        ];
        for transcode_command in self.get_commands(source, target) {
            lines.push(format!(
                "[pad=0|10|0|0]Transcode[/pad] [code]{transcode_command}[/code]"
            ));
        }
        let path = self.paths.get_transcode_target_dir(source, target);
        let factory = InspectFactory::new(false);
        let details = factory.create_split(&path);
        match details {
            Ok((properties, tags)) => {
                lines.push(format!(
                    "[pad=0|10|0|19]Details[/pad] [pre]{properties}[/pre]"
                ));
                lines.push(format!(
                    "[pad=0|10|0|31]Tags[/pad] [hide][pre]{tags}[/pre][/hide]"
                ));
            }
            Err(e) => {
                warn!(
                    "Unable to add track details to upload description\n{}",
                    e.render()
                );
            }
        }
        lines.into_iter().fold(String::new(), |mut output, line| {
            output.push_str("[quote]");
            output.push_str(&line);
            output.push_str("[/quote]");
            output
        })
    }

    /// Collect unique transcode commands for a source and target format.
    pub(crate) fn get_commands(&self, source: &Source, target: TargetFormat) -> HashSet<String> {
        let flacs = Collector::get_flacs(&source.directory);
        flacs
            .into_iter()
            .filter_map(|flac| {
                self.get_command_internal(flac, source, target)
                    .unwrap_or_else(|e| {
                        warn!("{}", e.render());
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
    ) -> Result<Option<String>, Failure<UploadAction>> {
        let job = self
            .transcode_job_factory
            .create_single(0, &flac, source, target)
            .map_err(Failure::wrap(UploadAction::GetTranscodeCommand))?;
        let Job::Transcode(job) = job else {
            unreachable!("TranscodeJobFactory::create_single always returns Job::Transcode")
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
}
