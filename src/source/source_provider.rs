use std::path::{Path, PathBuf};

use di::{injectable, Ref, RefMut};
use html_escape::decode_html_entities;

use crate::api::Api;
use crate::formats::ExistingFormatProvider;
use crate::imdl::imdl_command::ImdlCommand;
use crate::options::SharedOptions;
use crate::source::SourceError::*;
use crate::source::*;

/// Retrieve [Source] from the [Api] via a [provider design pattern](https://en.wikipedia.org/wiki/Provider_model)
#[injectable]
pub struct SourceProvider {
    api: RefMut<Api>,
    options: Ref<SharedOptions>,
}

impl SourceProvider {
    pub async fn get_by_string(&mut self, input: &String) -> Result<Source, SourceError> {
        if is_id_number(input) {
            let id = input.parse::<i64>().expect("ID should be a number");
            self.get_by_torrent_id(id).await
        } else if is_url(input) {
            self.get_by_url(input).await
        } else if is_torrent_file(input) {
            let path = PathBuf::from(input);
            if path.exists() {
                self.get_by_torrent_file(&path).await
            } else {
                Err(FileDoesNotExist(path.to_string_lossy().to_string()))
            }
        } else {
            Err(InvalidInput(input.clone()))
        }
    }

    async fn get_by_torrent_id(&mut self, id: i64) -> Result<Source, SourceError> {
        let mut api = self.api.write().expect("API should be available to read");
        let response = match api.get_torrent(id).await {
            Ok(response) => response,
            Err(error) => return Err(ApiFailure(error)),
        };
        let torrent = response.torrent;
        let group = response.group;
        let response = match api.get_torrent_group(group.id).await {
            Ok(response) => response,
            Err(error) => return Err(ApiFailure(error)),
        };
        if group.id != response.group.id {
            return Err(GroupMisMatch(group.id, response.group.id));
        }
        let group_torrents = response.torrents;
        let format = torrent.get_format().expect("Format should be valid");
        let format = format.to_source().ok_or(NotLossless(format))?;
        let existing =
            ExistingFormatProvider::get(&torrent, &group_torrents).expect("Format should be valid");
        let directory = self
            .options
            .content_directory
            .clone()
            .expect("Option should be set")
            .join(decode_html_entities(&torrent.file_path).to_string());
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

    async fn get_by_url(&mut self, url: &str) -> Result<Source, SourceError> {
        let base = &self
            .options
            .indexer_url
            .clone()
            .expect("Options should be set");
        let torrent_id = get_torrent_id_from_url(url, base).ok_or(TorrentIdNotFound)?;
        self.get_by_torrent_id(torrent_id).await
    }

    async fn get_by_torrent_file(&mut self, path: &Path) -> Result<Source, SourceError> {
        let summary = match ImdlCommand::show(path).await {
            Ok(summary) => summary,
            Err(error) => return Err(ImdlFailure(error)),
        };
        let tracker_id = self
            .options
            .indexer
            .clone()
            .expect("Options should be set")
            .to_uppercase();
        if summary.source != Some(tracker_id.clone()) {
            Err(SourceDoesNotMatch(
                tracker_id,
                summary.source.expect("Options should be set"),
            ))
        } else {
            let url = summary.comment.ok_or(TorrentIdNotFound)?;
            self.get_by_url(&url).await
        }
    }
}

fn is_url(input: &str) -> bool {
    input.starts_with("http")
}

fn is_torrent_file(input: &str) -> bool {
    input.ends_with(".torrent")
}

fn is_id_number(input: &str) -> bool {
    input.parse::<i64>().is_ok()
}
