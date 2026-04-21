use crate::testing_prelude::*;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::Torrent;
use qbittorrent_api::get_torrents::Torrent as QbitTorrent;

#[test]
fn queue_item_from_torrent_valid_data() {
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
    assert_eq!(result.indexer, Some(Indexer::Other("abc".to_owned())));
    assert_eq!(result.id, Some(12345));
}

#[test]
fn queue_item_from_torrent_missing_source() {
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
    assert_eq!(result.indexer, None);
}

#[test]
fn queue_item_from_torrent_missing_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = make_torrent("Example Torrent", Some("ABC"), None);

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert!(result.id.is_none());
}

#[test]
fn queue_item_from_torrent_invalid_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = make_torrent("Example Torrent", Some("Indexer"), Some("invalid_url"));

    // Act
    let result = QueueItem::from_torrent(path, &torrent);

    // Assert
    assert!(result.id.is_none());
}

#[test]
fn queue_item_from_qbit_torrent_red_comment() {
    // Arrange
    let torrent = QbitTorrent {
        comment: Some("https://redacted.sh/torrents.php?id=100&torrentid=200".to_owned()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(output.name, torrent.name);
    assert_eq!(
        output.hash,
        Hash::from_string(&torrent.hash).expect("hash should be valid")
    );
    assert_eq!(output.indexer, Some(Indexer::Red));
    assert_eq!(output.id, Some(200));
}

#[test]
fn queue_item_from_qbit_torrent_red_old_domain() {
    // Arrange
    let torrent = QbitTorrent {
        comment: Some("https://redacted.ch/torrents.php?torrentid=126".to_owned()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(output.indexer, Some(Indexer::Red));
    assert_eq!(output.id, Some(126));
}

#[test]
fn queue_item_from_qbit_torrent_ops_comment() {
    // Arrange
    let torrent = QbitTorrent {
        comment: Some("https://orpheus.network/torrents.php?id=300&torrentid=400".to_owned()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(output.indexer, Some(Indexer::Ops));
    assert_eq!(output.id, Some(400));
}

#[test]
fn queue_item_from_qbit_torrent_missing_comment() {
    // Arrange
    let torrent = QbitTorrent {
        comment: None,
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(output.indexer, None);
    assert!(output.id.is_none());
}

#[test]
fn queue_item_from_qbit_torrent_unknown_domain() {
    // Arrange
    let torrent = QbitTorrent {
        comment: Some("https://unknown.example/torrents.php?id=1&torrentid=2".to_owned()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(output.indexer, None);
    assert_eq!(output.id, Some(2));
}

#[test]
fn queue_item_from_qbit_torrent_invalid_hash() {
    // Arrange
    let torrent = QbitTorrent {
        hash: "not-a-valid-hex".to_owned(),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent);

    // Assert
    assert!(output.is_none());
}

/// For hybrid torrents qBittorrent's `hash` field is the truncated SHA-256 v2 info
/// hash, so `infohash_v1` must take precedence to recover the v1 SHA-1.
#[test]
fn queue_item_from_qbit_torrent_prefers_infohash_v1() {
    // Arrange
    let v1 = "1111111111111111111111111111111111111111";
    let torrent = QbitTorrent {
        hash: "2222222222222222222222222222222222222222".to_owned(),
        infohash_v1: Some(v1.to_owned()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(
        output.hash,
        Hash::from_string(v1).expect("hash should be valid")
    );
}

/// Empty `infohash_v1` (e.g. v2-only torrents serialized by qBittorrent) should
/// fall back to the `hash` field rather than failing on the empty string.
#[test]
fn queue_item_from_qbit_torrent_empty_infohash_v1_fallback() {
    // Arrange
    let torrent = QbitTorrent {
        infohash_v1: Some(String::new()),
        ..QbitTorrent::mock()
    };

    // Act
    let output = QueueItem::from_qbit_torrent(&torrent).expect("should produce item");

    // Assert
    assert_eq!(
        output.hash,
        Hash::from_string(&torrent.hash).expect("hash should be valid")
    );
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
