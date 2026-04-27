use crate::prelude::*;
use std::path::Component;

/// Retrieve [`Source`] from the API.
#[injectable]
pub struct SourceProvider {
    api: Ref<GazelleClient>,
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
    /// Includes content checks (source directory must exist locally).
    ///
    /// Returns:
    /// - `Ok(Ok(source))` - Source retrieved successfully
    /// - `Ok(Err(issue))` - Source not available (not found, missing directory, etc.)
    /// - `Err(failure)` - Operation failed (unauthorized, rate limited, network error)
    pub async fn get(&self, id: u32) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let mut source = match self.get_without_content(id).await? {
            Ok(source) => source,
            Err(issue) => return Ok(Err(issue)),
        };
        let directory = match self.get_source_directory(&source.torrent) {
            Ok(dir) => dir,
            Err(issue) => return Ok(Err(issue)),
        };
        source.directory = directory;
        Ok(Ok(source))
    }

    /// Retrieve a [`Source`] by torrent ID without content checks.
    ///
    /// Skips source directory lookup. The returned [`Source`] will have an
    /// empty `directory` field.
    ///
    /// Returns:
    /// - `Ok(Ok(source))` - Source retrieved successfully
    /// - `Ok(Err(issue))` - Source not available (not found, not a source format, etc.)
    /// - `Err(failure)` - Operation failed (unauthorized, rate limited, network error)
    pub async fn get_without_content(
        &self,
        id: u32,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let response = match self.api.get_torrent(id).await {
            Ok(response) => response,
            Err(e) if e.is_missing() => return Ok(Err(SourceIssue::NotFound)),
            Err(e) => return Err(Failure::new(SourceAction::GetTorrent, e)),
        };
        self.build_source(response).await
    }

    async fn get_without_content_by_hash(
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
        let metadata = Metadata::new(&group, &torrent);
        let url = get_permalink(&self.options.indexer_url, group.id, torrent.id);
        Ok(Ok(Source {
            torrent,
            group,
            targets,
            format,
            directory: PathBuf::new(),
            metadata,
            url,
        }))
    }

    fn get_source_directory(&self, torrent: &Torrent) -> Result<PathBuf, SourceIssue> {
        let path = torrent.file_path.as_str();
        let candidates = safe_candidates(path)?;
        self.find_directory(&candidates)
            .ok_or_else(|| SourceIssue::MissingDirectory {
                path: PathBuf::from(path),
            })
    }

    /// Find an existing directory in the configured content paths matching any candidate.
    ///
    /// - Joins each content path with each candidate and filters to existing directories.
    /// - Warns and returns the first when multiple match.
    /// - Returns `None` if no candidate matches on disk.
    fn find_directory(&self, candidates: &[String]) -> Option<PathBuf> {
        let directories: Vec<PathBuf> = self
            .options
            .content_paths()
            .iter()
            .flat_map(|x| candidates.iter().map(|p| x.join(p)))
            .filter(|x| x.is_dir())
            .collect();
        if directories.len() > 1 {
            warn!(
                "{} multiple content directories matching the torrent. The first will be used.",
                "Found".bold()
            );
            for directory in &directories {
                trace!("{}", directory.display());
            }
        }
        directories.into_iter().next()
    }

    /// Retrieve a [`Source`] using the ID from CLI options.
    pub async fn get_from_options(
        &self,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let mut source = match self.get_from_options_without_content().await? {
            Ok(source) => source,
            Err(issue) => return Ok(Err(issue)),
        };
        let directory = match self.get_source_directory(&source.torrent) {
            Ok(dir) => dir,
            Err(issue) => return Ok(Err(issue)),
        };
        source.directory = directory;
        Ok(Ok(source))
    }

    /// Retrieve a [`Source`] using the ID from CLI options without content checks.
    ///
    /// Skips source directory lookup. The returned [`Source`] will have an
    /// empty `directory` field.
    pub async fn get_from_options_without_content(
        &self,
    ) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        if let Some(hash) = self.hash_from_arg() {
            return self.get_without_content_by_hash(hash).await;
        }
        let id = self
            .id_provider
            .get_by_options()
            .await
            .map_err(SourceIssue::Id);
        match id {
            Ok(id) => self.get_without_content(id).await,
            Err(issue) => Ok(Err(issue)),
        }
    }

    fn hash_from_arg(&self) -> Option<&str> {
        let source = self.arg.source.as_str();
        HASH_PATTERN.is_match(source).then_some(source)
    }
}

/// Validate `path` and produce content-path lookup candidates.
///
/// - Returns [`SourceIssue::NoDirectory`] for an empty `path`.
/// - Runs libtorrent and invisible sanitizers, warning on stripped characters.
/// - Returns [`SourceIssue::InvalidFilePath`] if any candidate is not a single safe segment.
/// - Deduplicates while preserving raw-first order.
fn safe_candidates(path: &str) -> Result<Vec<String>, SourceIssue> {
    if path.is_empty() {
        return Err(SourceIssue::NoDirectory);
    }
    let libtorrent = Sanitizer::libtorrent().execute(path.to_owned());
    let invisible = Sanitizer::invisible().execute(path.to_owned());
    let mut found: HashSet<SanitizerChar> = HashSet::new();
    found.extend(libtorrent.found.iter().copied());
    found.extend(invisible.found.iter().copied());
    if !found.is_empty() {
        let chars: Vec<&SanitizerChar> = found.iter().collect();
        warn!(
            "Invisible characters in source path: {}",
            join_humanized(chars)
        );
    }
    let candidates = [path.to_owned(), libtorrent.output, invisible.output];
    if !candidates.iter().all(|c| is_single_safe_segment(c)) {
        return Err(SourceIssue::InvalidFilePath {
            path: path.to_owned(),
        });
    }
    let mut unique: Vec<String> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !unique.contains(&candidate) {
            unique.push(candidate);
        }
    }
    Ok(unique)
}

/// Whether `path` is a single normal path segment with no backslash.
///
/// - Empty or pure-whitespace strings return `false`.
/// - Backslashes are rejected explicitly so behavior matches across Linux and Windows
///   (Windows treats `\` as a path separator, Linux does not).
/// - Anything other than exactly one [`Component::Normal`] returns `false`.
fn is_single_safe_segment(path: &str) -> bool {
    if path.contains('\\') {
        return false;
    }
    if path.trim().is_empty() {
        return false;
    }
    let mut components = Path::new(path).components();
    let Some(Component::Normal(_)) = components.next() else {
        return false;
    };
    components.next().is_none()
}
