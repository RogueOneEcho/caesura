use std::path::{Path, PathBuf};

use di::{injectable, Ref, RefMut};
use html_escape::decode_html_entities;

use crate::api::Api;
use crate::errors::AppError;
use crate::formats::ExistingFormatProvider;
use crate::imdl::imdl_command::ImdlCommand;
use crate::options::SharedOptions;
use crate::source::*;

/// Retrieve [Source] from the [Api] via a [provider design pattern](https://en.wikipedia.org/wiki/Provider_model)
#[injectable]
pub struct SourceProvider {
    api: RefMut<Api>,
    options: Ref<SharedOptions>,
}

impl SourceProvider {
    pub async fn get_by_string(&mut self, input: &String) -> Result<Source, AppError> {
        if is_id_number(input) {
            let id = input.parse::<i64>().expect("ID should be a number");
            self.get_by_id(id).await
        } else if is_url(input) {
            self.get_by_url(input).await
        } else if is_torrent_file(input) {
            let path = PathBuf::from(input);
            if path.exists() {
                self.get_by_file(&path).await
            } else {
                AppError::explained(
                    "get source from torrent file",
                    "File does not exist".to_owned(),
                )
            }
        } else {
            AppError::explained("get source", format!("Unknown source: {input}"))
        }
    }

    async fn get_by_id(&mut self, id: i64) -> Result<Source, AppError> {
        let action = "get source by id";
        let mut api = self.api.write().expect("API should be available to read");
        let response = api.get_torrent(id).await?;
        let torrent = response.torrent;
        let group = response.group;
        let response = api.get_torrent_group(group.id).await?;
        if group.id != response.group.id {
            AppError::explained(
                action,
                "group of torrent did not match torrent group".to_owned(),
            )?;
        }
        let group_torrents = response.torrents;
        let format = torrent.get_format()?.to_source()?;
        let existing = ExistingFormatProvider::get(&torrent, &group_torrents)?;
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

    async fn get_by_url(&mut self, url: &str) -> Result<Source, AppError> {
        let base = &self
            .options
            .indexer_url
            .clone()
            .expect("Options should be set");
        let torrent_id = get_torrent_id_from_url(url, base)?;
        self.get_by_id(torrent_id).await
    }

    async fn get_by_file(&mut self, path: &Path) -> Result<Source, AppError> {
        let action = "get source by file";
        let summary = ImdlCommand::show(path).await?;
        let tracker_id = self
            .options
            .indexer
            .clone()
            .expect("Options should be set")
            .to_uppercase();
        if summary.source == Some(tracker_id.clone()) {
            let url = summary.comment.unwrap_or_default();
            if is_url(url.as_str()) {
                self.get_by_url(&url).await
            } else {
                AppError::explained(action, "comment is not a url".to_owned())
            }
        } else {
            AppError::unexpected(
                action,
                "incorrect source",
                tracker_id,
                summary.source.unwrap_or_default(),
            )
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
