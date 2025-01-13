use crate::options::*;
use crate::utils::*;

use colored::Colorize;
use di::{injectable, Ref, RefMut};
use gazelle_api::{GazelleClient, Torrent};
use html_escape::decode_html_entities;
use log::{trace, warn};
use std::path::PathBuf;

/// Retrieve [Source] from the [Api] via a [provider design pattern](https://en.wikipedia.org/wiki/Provider_model)
#[injectable]
pub struct SourceProvider {
    api: RefMut<GazelleClient>,
    options: Ref<SharedOptions>,
    id_provider: Ref<IdProvider>,
}

impl SourceProvider {
    pub async fn get(&mut self, id: u32) -> Result<Source, SourceIssue> {
        let mut api = self.api.write().expect("API should be available to read");
        let response = api.get_torrent(id).await.map_err(SourceIssue::api)?;
        let torrent = response.torrent;
        let group = response.group;
        let response = api
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
        let directories: Vec<PathBuf> = self
            .options
            .content
            .clone()
            .expect("content should be set")
            .iter()
            .map(|x| x.join(path.clone()))
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

    pub async fn get_from_options(&mut self) -> Result<Source, SourceIssue> {
        let id = self
            .id_provider
            .get_by_options()
            .await
            .map_err(SourceIssue::Id)?;
        self.get(id).await
    }
}
