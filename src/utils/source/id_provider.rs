use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::dependencies::*;
use crate::options::*;
use crate::utils::IdProviderError::*;
use crate::utils::*;
use di::{Ref, injectable};
use log::warn;
use serde::{Deserialize, Serialize};

/// Retrieve the id of a source.
#[injectable]
pub struct IdProvider {
    options: Ref<SharedOptions>,
    arg: Ref<SourceArg>,
}

/// Error types returned by the [`IdProvider`]
///
/// ## Notes
/// In v0.24.0 the keys were serialized as `PascalCase`.
/// In v0.24.1 behaviour was fixed to serialize as `snake_case` therefore `alias` are necessary
/// to ensure backwards compatibility.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdProviderError {
    /// No ID was provided
    NoId,
    /// Input did not match any known types
    #[serde(alias = "NoMatch")]
    NoMatch,
    /// Input was a URL that could not be parsed
    #[serde(alias = "UrlInvalid")]
    UrlInvalid,
    /// Input was a torrent file that did not exist
    #[serde(alias = "TorrentFileNotFound")]
    TorrentFileNotFound,
    /// Input was a torrent file that IMDL failed to show
    #[serde(alias = "TorrentFileInvalid")]
    TorrentFileInvalid,
    /// Input was a torrent file with an unwanted source
    #[serde(alias = "TorrentFileSource")]
    TorrentFileSource { actual: String, expected: String },
}

impl IdProvider {
    /// Get source ID from CLI options.
    pub async fn get_by_options(&self) -> Result<u32, IdProviderError> {
        let source_input = self.arg.source.clone().unwrap_or_default();
        self.get_by_string(&source_input).await
    }

    async fn get_by_string(&self, input: &String) -> Result<u32, IdProviderError> {
        if let Ok(id) = input.parse::<u32>() {
            Ok(id)
        } else if input.starts_with("http") {
            get_torrent_id_from_url(input)
        } else if input.ends_with(".torrent") {
            let path = PathBuf::from(input);
            if path.exists() {
                self.get_by_file(&path).await
            } else {
                Err(TorrentFileNotFound)
            }
        } else {
            Err(NoMatch)
        }
    }

    async fn get_by_file(&self, path: &Path) -> Result<u32, IdProviderError> {
        let summary = ImdlCommand::show(path).await.map_err(|e| {
            warn!("{e}");
            TorrentFileInvalid
        })?;
        let tracker_id = self.options.indexer.clone().expect("indexer should be set");
        if summary.is_source_equal(&tracker_id) {
            let url = summary.comment.unwrap_or_default();
            get_torrent_id_from_url(&url)
        } else {
            Err(TorrentFileSource {
                actual: summary.source.unwrap_or_default(),
                expected: tracker_id.to_ascii_uppercase(),
            })
        }
    }
}

impl Display for IdProviderError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            NoId => "No ID was provided".to_owned(),
            NoMatch => "Input did not match any known types".to_owned(),
            UrlInvalid => "Input was a URL that could not be parsed".to_owned(),
            TorrentFileNotFound => "Input was a torrent file that did not exist".to_owned(),
            TorrentFileInvalid => "Input was a torrent file that could not be read".to_owned(),
            TorrentFileSource { actual, expected } => {
                format!("Input was a torrent file with source {actual} not {expected}")
            }
        };
        message.fmt(formatter)
    }
}
