use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub private: bool,
    pub source: bool,
    #[serde(rename = "requestid")]
    pub request_id: Option<i64>,
    torrentid: Option<i64>,
    groupid: Option<i64>,
    torrentId: Option<i64>,
    groupId: Option<i64>,
}

impl UploadResponse {
    pub fn get_torrent_id(&self) -> i64 {
        self.torrentid.unwrap_or_else(|| self.torrentId.unwrap_or_default())
    }
    pub fn get_group_id(&self) -> i64 {
        self.groupid.unwrap_or_else(|| self.groupId.unwrap_or_default())
    }
}
