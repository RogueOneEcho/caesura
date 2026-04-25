use crate::prelude::*;

/// Retrieve [`Source`] from the API.
#[injectable]
pub struct SourceProvider {
    api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    arg: Ref<SourceArg>,
    options: Ref<SharedOptions>,
    id_provider: Ref<IdProvider>,
    existing_provider: Ref<ExistingFormatProvider>,
    target_provider: Ref<TargetFormatProvider>,
}

static HASH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-fA-F]{40}$").expect("regex should compile"));

impl SourceProvider {
    /// Retrieve a [`Source`] by torrent ID.
    ///
    /// Returns:
    /// - `Ok(Ok(source))` - Source retrieved successfully
    /// - `Ok(Err(issue))` - Source not available (not found, missing directory, etc.)
    /// - `Err(failure)` - Operation failed (unauthorized, rate limited, network error)
    pub async fn get(&self, id: u32) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let response = match self.api.get_torrent(id).await {
            Ok(response) => response,
            Err(e) if e.is_missing() => return Ok(Err(SourceIssue::NotFound)),
            Err(e) => return Err(Failure::new(SourceAction::GetTorrent, e)),
        };
        self.build_source(response).await
    }

    async fn get_by_hash(
        &self,
        hash: &str,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let response = match self.api.get_torrent_by_hash(&hash.to_uppercase()).await {
            Ok(response) => response,
            Err(e) if e.is_missing() => return Ok(Err(SourceIssue::NotFound)),
            Err(e) => return Err(Failure::new(SourceAction::GetTorrent, e)),
        };
        self.build_source(response).await
    }

    fn hash_from_arg(&self) -> Option<&str> {
        let source = self.arg.source.as_str();
        HASH_PATTERN.is_match(source).then_some(source)
    }

    async fn build_source(
        &self,
        response: TorrentResponse,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let torrent = response.torrent;
        let group = response.group;
        let response = match self.api.get_torrent_group(group.id).await {
            Ok(response) => response,
            Err(e) if e.is_missing() => return Ok(Err(SourceIssue::NotFound)),
            Err(e) => return Err(Failure::new(SourceAction::GetTorrentGroup, e)),
        };
        if group.id != response.group.id {
            return Ok(Err(SourceIssue::GroupMismatch {
                expected: group.id,
                actual: response.group.id,
            }));
        }
        let group_torrents = response.torrents;
        let Some(format) =
            ExistingFormat::from_torrent(&torrent).and_then(ExistingFormat::to_source)
        else {
            return Ok(Err(SourceIssue::NotSource {
                format: torrent.format.to_string(),
                encoding: torrent.encoding.to_string(),
            }));
        };
        let existing = self.existing_provider.get(&torrent, &group_torrents);
        let targets = self.target_provider.get(format, &existing);
        let directory = match self.get_source_directory(&torrent) {
            Ok(dir) => dir,
            Err(issue) => return Ok(Err(issue)),
        };
        let metadata = Metadata::new(&group, &torrent);
        let url = get_permalink(&self.options.indexer_url, group.id, torrent.id);
        Ok(Ok(Source {
            torrent,
            group,
            targets,
            format,
            directory,
            metadata,
            url,
        }))
    }

    fn get_source_directory(&self, torrent: &Torrent) -> Result<PathBuf, SourceIssue> {
        let path = decode_html_entities(&torrent.file_path).to_string();
        let result = Sanitizer::libtorrent().execute(path.clone());
        if !result.found.is_empty() {
            warn!("Invisible characters in source path: {}", result.humanize());
        }
        let safe_path = result.output;
        let mut paths = vec![&path];
        if safe_path != path {
            paths.push(&safe_path);
        }
        let directories: Vec<PathBuf> = self
            .options
            .content_paths()
            .iter()
            .flat_map(|x| paths.iter().map(|p| x.join(p)))
            .filter(|x| x.is_dir())
            .collect();
        if directories.is_empty() {
            return Err(SourceIssue::MissingDirectory {
                path: PathBuf::from(path),
            });
        } else if directories.len() > 1 {
            warn!(
                "{} multiple content directories matching the torrent. The first will be used.",
                "Found".bold()
            );
            for directory in &directories {
                trace!("{}", directory.display());
            }
        }
        Ok(directories.first().expect("should be at least one").clone())
    }

    /// Retrieve a [`Source`] using the ID from CLI options.
    pub async fn get_from_options(
        &self,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        if let Some(hash) = self.hash_from_arg() {
            return self.get_by_hash(hash).await;
        }
        let id = self
            .id_provider
            .get_by_options()
            .await
            .map_err(SourceIssue::Id);
        match id {
            Ok(id) => self.get(id).await,
            Err(issue) => Ok(Err(issue)),
        }
    }
}
