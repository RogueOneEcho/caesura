use crate::testing_prelude::*;
use flat_db::Hash;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::Torrent;
use std::collections::HashMap;

#[test]
fn from_torrent_with_valid_data() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let name = "Artist - Album (2018) [FLAC]";
    let source = "ABC";
    let comment = "https://example.com/torrents.php?torrentid=12345";
    let torrent = make_torrent(name, Some(source), Some(comment));
    let info_hash = torrent.info_hash();

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert_eq!(result.name, name);
    assert_eq!(
        result.hash,
        Hash::from_string(&info_hash).expect("hash should be valid")
    );
    assert_eq!(result.indexer, source.to_lowercase());
    assert_eq!(result.id, Some(12345));
}

#[test]
fn from_torrent_with_missing_source() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = make_torrent(
        "Example Torrent",
        None,
        Some("https://example.com/torrents.php?torrentid=12345"),
    );

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert!(result.indexer.is_empty());
}

#[test]
fn from_torrent_with_missing_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = make_torrent("Example Torrent", Some("ABC"), None);

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert!(result.id.is_none());
}

#[test]
fn from_torrent_with_invalid_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = make_torrent("Example Torrent", Some("Indexer"), Some("invalid_url"));

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert!(result.id.is_none());
}

fn make_torrent(name: &str, source: Option<&str>, comment: Option<&str>) -> Torrent {
    let extra_info_fields = source.map(|s| {
        let mut map = HashMap::new();
        map.insert("source".to_owned(), BencodeElem::String(s.to_owned()));
        map
    });
    let extra_fields = comment.map(|c| {
        let mut map = HashMap::new();
        map.insert("comment".to_owned(), BencodeElem::String(c.to_owned()));
        map
    });
    Torrent {
        announce: None,
        announce_list: None,
        length: 0,
        files: None,
        name: name.to_owned(),
        piece_length: 16384,
        pieces: Vec::new(),
        extra_fields,
        extra_info_fields,
    }
}
