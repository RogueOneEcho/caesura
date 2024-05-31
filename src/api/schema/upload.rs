use reqwest::multipart::{Form, Part};

use crate::api::Category;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentUploadData {
    pub torrent: Vec<u8>,
    pub torrent_name: String,
    pub r#type: Category,
    pub remaster_year: i64,
    pub remaster_title: String,
    pub remaster_record_label: String,
    pub remaster_catalogue_number: String,
    pub format: String,
    pub bitrate: String,
    pub media: String,
    pub release_desc: String,
    pub group_id: u64,
}

impl From<TorrentUploadData> for Form {
    fn from(val: TorrentUploadData) -> Self {
        let torrent_part = Part::bytes(val.torrent).file_name(val.torrent_name);
        Form::new()
            .part("file_input", torrent_part)
            .text("type", val.r#type.as_int().to_string())
            .text("remaster_title", val.remaster_title)
            .text("remaster_record_label", val.remaster_record_label)
            .text("remaster_catalogue_number", val.remaster_catalogue_number)
            .text("remaster_year", val.remaster_year.to_string())
            .text("format", val.format)
            .text("bitrate", val.bitrate)
            .text("media", val.media)
            .text("release_desc", val.release_desc)
            .text("groupid", val.group_id.to_string())
    }
}
