use std::path::{Path, PathBuf};

use di::{injectable, Ref};

use rogue_logging::Error;

use crate::dependencies::*;
use crate::options::*;
use crate::utils::*;
/// Retrieve the id of a source.
#[injectable]
pub struct IdProvider {
    options: Ref<SharedOptions>,
    arg: Ref<SourceArg>,
}

impl IdProvider {
    pub async fn get_by_options(&self) -> Result<u32, Error> {
        let source_input = self.arg.source.clone().unwrap_or_default();
        self.get_by_string(&source_input).await
    }

    pub async fn get_by_string(&self, input: &String) -> Result<u32, Error> {
        if let Ok(id) = input.parse::<u32>() {
            Ok(id)
        } else if input.starts_with("http") {
            get_torrent_id_from_url(input)
        } else if input.ends_with(".torrent") {
            let path = PathBuf::from(input);
            if path.exists() {
                self.get_by_file(&path).await
            } else {
                Err(error(
                    "get source from torrent file",
                    "File does not exist".to_owned(),
                ))
            }
        } else {
            Err(error("get source", format!("Unknown source: {input}")))
        }
    }

    async fn get_by_file(&self, path: &Path) -> Result<u32, Error> {
        let summary = ImdlCommand::show(path).await?;
        let tracker_id = self.options.indexer.clone().expect("indexer should be set");
        if summary.is_source_equal(&tracker_id) {
            let url = summary.comment.unwrap_or_default();
            get_torrent_id_from_url(&url)
        } else {
            Err(error(
                "get source by file",
                format!(
                    "incorrect source\nExpected: {tracker_id}\nActual: {}",
                    summary.source.unwrap_or_default()
                ),
            ))
        }
    }
}
