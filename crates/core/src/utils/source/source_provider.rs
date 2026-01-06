use crate::prelude::*;
use gazelle_api::{GazelleClientTrait, Torrent};
use html_escape::decode_html_entities;

/// Retrieve [`Source`] from the API.
#[injectable]
pub struct SourceProvider {
    api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    options: Ref<SharedOptions>,
    id_provider: Ref<IdProvider>,
}

fn libtorrent_safe_path(path: &str) -> String {
    // https://github.com/arvidn/libtorrent/blob/9c1897645265c6a450930e766ab46c02a240891f/src/torrent_info.cpp#L100
    path.replace(
        [
            '/', '\\', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
            '\u{202e}',
        ],
        "",
    )
}

impl SourceProvider {
    /// Retrieve a [`Source`] by torrent ID.
    pub async fn get(&self, id: u32) -> Result<Source, SourceIssue> {
        let response = self.api.get_torrent(id).await.map_err(SourceIssue::api)?;
        let torrent = response.torrent;
        let group = response.group;
        let response = self
            .api
            .get_torrent_group(group.id)
            .await
            .map_err(SourceIssue::api)?;
        if group.id != response.group.id {
            return Err(SourceIssue::GroupMismatch {
                actual: group.id,
                expected: response.group.id,
            });
        }
        let group_torrents = response.torrents;
        let Some(format) =
            ExistingFormat::from_torrent(&torrent).and_then(ExistingFormat::to_source)
        else {
            return Err(SourceIssue::NotSource {
                format: torrent.format,
                encoding: torrent.encoding,
            });
        };
        let existing = ExistingFormatProvider::get(&torrent, &group_torrents);
        let directory = self.get_source_directory(&torrent)?;
        let metadata = Metadata::new(&group, &torrent);
        Ok(Source {
            torrent,
            group,
            existing,
            format,
            directory,
            metadata,
        })
    }

    fn get_source_directory(&self, torrent: &Torrent) -> Result<PathBuf, SourceIssue> {
        let path = decode_html_entities(&torrent.file_path).to_string();
        let safe_path = libtorrent_safe_path(&path);

        let mut paths = vec![&path, &safe_path];
        paths.dedup();

        let directories: Vec<PathBuf> = self
            .options
            .content
            .iter()
            .flat_map(|x| paths.iter().map(|p| x.join((*p).clone())))
            .filter(|x| x.exists() && x.is_dir())
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
    pub async fn get_from_options(&self) -> Result<Source, SourceIssue> {
        let id = self
            .id_provider
            .get_by_options()
            .await
            .map_err(SourceIssue::Id)?;
        self.get(id).await
    }
}
