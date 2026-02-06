use crate::prelude::*;
use gazelle_api::{ApiResponseKind, GazelleClientTrait, GazelleOperation, Torrent};
use html_escape::decode_html_entities;

/// Retrieve [`Source`] from the API.
#[injectable]
pub struct SourceProvider {
    api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    options: Ref<SharedOptions>,
    id_provider: Ref<IdProvider>,
}

impl SourceProvider {
    /// Retrieve a [`Source`] by torrent ID.
    ///
    /// Returns:
    /// - `Ok(Ok(source))` - Source retrieved successfully
    /// - `Ok(Err(issue))` - Source not available (not found, missing directory, etc.)
    /// - `Err(failure)` - Operation failed (unauthorized, rate limited, network error)
    pub async fn get(&self, id: u32) -> Result<Result<Source, SourceIssue>, Failure<SourceAction>> {
        let not_found = GazelleOperation::ApiResponse(ApiResponseKind::NotFound);
        let bad_request = GazelleOperation::ApiResponse(ApiResponseKind::BadRequest);
        let response = match self.api.get_torrent(id).await {
            Ok(response) => response,
            Err(e) if e.operation == not_found || e.operation == bad_request => {
                return Ok(Err(SourceIssue::NotFound));
            }
            Err(e) => return Err(Failure::new(SourceAction::GetTorrent, e)),
        };
        let torrent = response.torrent;
        let group = response.group;
        let response = match self.api.get_torrent_group(group.id).await {
            Ok(response) => response,
            Err(e) if e.operation == not_found || e.operation == bad_request => {
                return Ok(Err(SourceIssue::NotFound));
            }
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
                format: torrent.format,
                encoding: torrent.encoding,
            }));
        };
        let existing = ExistingFormatProvider::get(&torrent, &group_torrents);
        let directory = match self.get_source_directory(&torrent) {
            Ok(dir) => dir,
            Err(issue) => return Ok(Err(issue)),
        };
        let metadata = Metadata::new(&group, &torrent);
        Ok(Ok(Source {
            torrent,
            group,
            existing,
            format,
            directory,
            metadata,
        }))
    }

    fn get_source_directory(&self, torrent: &Torrent) -> Result<PathBuf, SourceIssue> {
        let path = decode_html_entities(&torrent.file_path).to_string();
        let safe_path = libtorrent_safe_path(&path);
        let mut paths = vec![&path];
        if safe_path != path {
            paths.push(&safe_path);
        }
        let directories: Vec<PathBuf> = self
            .options
            .content
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

/// Strip characters that libtorrent removes from torrent file paths.
///
/// - <https://github.com/arvidn/libtorrent/blob/9c1897645265c6a450930e766ab46c02a240891f/src/torrent_info.cpp#L100>
fn libtorrent_safe_path(path: &str) -> String {
    path.replace(
        [
            '/', '\\', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
            '\u{202e}',
        ],
        "",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _libtorrent_safe_path() {
        assert_eq!(libtorrent_safe_path("no change"), "no change");
        assert_eq!(libtorrent_safe_path("AC/DC"), "ACDC");
        assert_eq!(libtorrent_safe_path(r"back\slash"), "backslash");
        assert_eq!(libtorrent_safe_path("a\u{200e}b\u{200f}c"), "abc");
        assert_eq!(
            libtorrent_safe_path("Hello\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}World"),
            "HelloWorld"
        );
        assert_eq!(libtorrent_safe_path(""), "");
    }
}
