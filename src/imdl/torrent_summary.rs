use serde::Deserialize;

/// Summary of a torrent file
///
/// <https://github.com/casey/intermodal/blob/master/src/torrent_summary.rs>
#[derive(Default, Deserialize)]
#[allow(dead_code)]
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

impl TorrentSummary {
    pub fn is_source_equal(&self, source: &str) -> bool {
        match self.source.clone() {
            None => false,
            Some(torrent) => source.eq_ignore_ascii_case(&torrent) || red_match_pth(source, &torrent),
        }
    }
}

fn red_match_pth(source: &str, torrent: &str) -> bool {
    source.eq_ignore_ascii_case("red") && torrent.eq_ignore_ascii_case("pth")
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn is_source_equal() {
        assert!(test("ops", "ops"));
        assert!(test("pth", "pth"));
        assert!(test("red", "red"));
        assert!(test("red", "RED"));
        assert!(test("RED", "RED"));
        assert!(test("red", "pth"));
        assert!(test("red", "PTH"));
        assert!(test("abc", "AbC"));
        assert!(!test("red", "ops"));
        assert!(!test("red", "OPS"));
        assert!(!test("red", "OPS"));
        assert!(!test("RED", "OPS"));
        assert!(!test("pth", "red"));
    }

    fn test(source: &str, torrent_source: &str) -> bool {
        let torrent = TorrentSummary {
            source: Some(torrent_source.to_owned()),
            ..TorrentSummary::default()
        };        
        torrent.is_source_equal(source)
    }
}