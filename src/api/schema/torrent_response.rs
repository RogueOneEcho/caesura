use crate::api::{Group, Torrent};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentResponse {
    pub group: Group,
    pub torrent: Torrent,
}