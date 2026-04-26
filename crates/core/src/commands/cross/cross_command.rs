use crate::prelude::*;

/// Find a source on the primary indexer and look up cross-seeds on the cross indexer.
#[injectable]
pub(crate) struct CrossCommand {
    cross: Ref<Option<CrossServices>>,
    cross_config_options: Ref<CrossConfigOptions>,
    cross_options: Ref<CrossOptions>,
    injector: Ref<TorrentInjector>,
    qbit_cross_options: Ref<QbitCrossOptions>,
    qbit_options: Ref<QbitOptions>,
    source_provider: Ref<SourceProvider>,
    torrent_files: Ref<TorrentFileProvider>,
}

impl CrossCommand {
    /// Execute [`CrossCommand`] from the CLI.
    ///
    /// Returns `true` if a cross-seed was found (and, unless `dry_run`, processed),
    /// `false` if no cross-seed was found or the source could not be resolved.
    pub(crate) async fn execute_cli(&self) -> Result<bool, Failure<CrossCommandAction>> {
        self.validate()?;
        let source = match self
            .source_provider
            .get_from_options_without_content()
            .await
        {
            Ok(Ok(source)) => source,
            Ok(Err(issue)) => {
                warn!("{} for cross seeding unknown", "Unsuitable".bold());
                warn!("{issue}");
                return Ok(false);
            }
            Err(e) => return Err(Failure::new(CrossCommandAction::GetSource, e)),
        };
        let main_torrent = self
            .torrent_files
            .get(source.torrent.id)
            .await
            .map_err(Failure::wrap(CrossCommandAction::DownloadSourceTorrent))?;
        let cross = self
            .cross
            .as_ref()
            .as_ref()
            .expect("cross config validated above so CrossServices must be Some");
        let cross_id = cross
            .checker
            .execute(&main_torrent, &source)
            .await
            .map_err(Failure::wrap(CrossCommandAction::CheckCrossIndexer))?;
        let Some(cross_id) = cross_id else {
            info!("{} cross seed for {source}", "No".bold());
            return Ok(false);
        };
        if self.cross_options.dry_run {
            info!(
                "{} inject cross seed {cross_id} for {source}",
                "Would".bold()
            );
            return Ok(true);
        }
        let cross_path = cross
            .torrent_files
            .get(cross_id)
            .await
            .map_err(Failure::wrap(CrossCommandAction::DownloadCrossTorrent))?;
        if let Some(target_dir) = &self.cross_options.copy_cross_torrent_to {
            self.injector
                .copy_torrent(&cross_path, target_dir)
                .await
                .map_err(Failure::wrap(CrossCommandAction::CopyTorrent))?;
        }
        if self.qbit_cross_options.qbit_cross {
            let add_options = self.qbit_cross_options.to_add_torrent_options();
            self.injector
                .inject_qbit(&cross_path, add_options)
                .await
                .map_err(Failure::wrap(CrossCommandAction::InjectTorrent))?;
            info!("{} cross seed {cross_id} for {source}", "Injected".bold());
        } else {
            info!("{} cross seed {cross_id} for {source}", "Found".bold());
        }
        Ok(true)
    }

    fn validate(&self) -> Result<(), Failure<CrossCommandAction>> {
        let mut validator = OptionsValidator::new();
        validator.check_set("cross_config", &self.cross_config_options.cross_config);
        if self.qbit_cross_options.qbit_cross {
            self.qbit_options.validate_connection(&mut validator);
        }
        if !self.cross_options.dry_run
            && !self.qbit_cross_options.qbit_cross
            && self.cross_options.copy_cross_torrent_to.is_none()
        {
            validator.push(OptionIssue::required_one_of(&[
                "qbit_cross",
                "copy_cross_torrent_to",
                "dry_run",
            ]));
        }
        validator.check_or(CrossCommandAction::ValidateOptions)
    }
}

/// Error action for [`CrossCommand`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum CrossCommandAction {
    /// Validate required options before running.
    #[error("validate options")]
    ValidateOptions,
    /// Retrieve the source from the API.
    #[error("get source")]
    GetSource,
    /// Download the source `.torrent` file from the primary indexer.
    #[error("download source torrent file")]
    DownloadSourceTorrent,
    /// Look up the source on the cross indexer.
    #[error("check cross indexer")]
    CheckCrossIndexer,
    /// Download the cross-seed `.torrent` file from the cross indexer.
    #[error("download cross torrent file")]
    DownloadCrossTorrent,
    /// Inject the cross-seed torrent into qBittorrent.
    #[error("inject torrent")]
    InjectTorrent,
    /// Copy the cross-seed torrent file to the autowatch directory.
    #[error("copy torrent file")]
    CopyTorrent,
}
