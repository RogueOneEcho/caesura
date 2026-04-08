//! Extension trait for convenient access to `lava_torrent` torrent fields.

use crate::prelude::Indexer;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::Torrent;
use std::collections::HashMap;

type Dictionary = HashMap<String, BencodeElem>;

/// Convenient access to `lava_torrent` torrent fields stored in extra dictionaries.
pub(crate) trait TorrentExt {
    /// Source tag (e.g. "RED", "OPS") from the info dict.
    fn source(&self) -> Option<&str>;
    /// Comment field from the top-level dict.
    fn comment(&self) -> Option<&str>;
    /// `created by` field from the top-level dict (e.g. "caesura v1.0.0").
    #[allow(
        dead_code,
        reason = "trait accessor used in tests; symmetric with comment()"
    )]
    fn created_by(&self) -> Option<&str>;
    /// Check if the torrent source matches, with RED/PTH equivalence.
    fn is_source_equal(&self, source: &Indexer) -> bool;
}

impl TorrentExt for Torrent {
    fn source(&self) -> Option<&str> {
        get_string(self.extra_info_fields.as_ref(), "source")
    }

    fn comment(&self) -> Option<&str> {
        get_string(self.extra_fields.as_ref(), "comment")
    }

    fn created_by(&self) -> Option<&str> {
        get_string(self.extra_fields.as_ref(), "created by")
    }

    fn is_source_equal(&self, source: &Indexer) -> bool {
        let Some(torrent_source) = self.source() else {
            return false;
        };
        let torrent_source = Indexer::from(torrent_source);
        source.match_with_alts(&torrent_source)
    }
}

fn get_string<'a>(dict: Option<&'a Dictionary>, key: &str) -> Option<&'a str> {
    dict.and_then(|d| match d.get(key) {
        Some(BencodeElem::String(s)) => Some(s.as_str()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn torrent_with_source(source: Option<&str>) -> Torrent {
        let extra_info_fields = source.map(|s| {
            let mut map = HashMap::new();
            map.insert("source".to_owned(), BencodeElem::String(s.to_owned()));
            map
        });
        Torrent {
            announce: None,
            announce_list: None,
            length: 0,
            files: None,
            name: String::new(),
            piece_length: 16384,
            pieces: Vec::new(),
            extra_fields: None,
            extra_info_fields,
        }
    }

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
        assert!(!test("RED", "OPS"));
        assert!(!test("pth", "red"));
    }

    fn test(source: &str, torrent_source: &str) -> bool {
        let torrent = torrent_with_source(Some(torrent_source));
        let source = Indexer::from(source);
        torrent.is_source_equal(&source)
    }
}
