use serde::Deserialize;

/// Summary of a torrent file
///
/// <https://github.com/casey/intermodal/blob/master/src/torrent_summary.rs>
#[derive(Deserialize)]
pub struct TorrentSummary {
    pub name: String,
    pub comment: Option<String>,
    pub creation_date: Option<u64>,
    pub created_by: Option<String>,
    pub source: Option<String>,
    pub info_hash: String,
    pub torrent_size: u64,
    pub content_size: u64,
    pub private: bool,
    pub tracker: Option<String>,
    pub announce_list: Vec<Vec<String>>,
    pub update_url: Option<String>,
    pub dht_nodes: Vec<String>,
    pub piece_size: u64,
    pub piece_count: usize,
    pub file_count: usize,
    pub files: Vec<String>,
}
