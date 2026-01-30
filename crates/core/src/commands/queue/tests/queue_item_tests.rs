use crate::testing_prelude::*;
use flat_db::Hash;

#[test]
fn from_torrent_with_valid_data() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let name = "Artist - Album (2018) [FLAC]".to_owned();
    let info_hash = "abcdef1234567890abcdef1234567890abcdef12".to_owned();
    let source = "ABC".to_owned();
    let comment = "https://example.com/torrents.php?torrentid=12345".to_owned();
    let torrent = TorrentSummary {
        name: name.clone(),
        info_hash: info_hash.clone(),
        source: Some(source.clone()),
        comment: Some(comment.clone()),
        ..TorrentSummary::default()
    };

    // Act
    let result = QueueItem::from_torrent(path, torrent);

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
    let torrent = TorrentSummary {
        name: "Example Torrent".to_owned(),
        info_hash: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
        source: None,
        comment: Some("https://example.com/torrents.php?torrentid=12345".to_owned()),
        ..TorrentSummary::default()
    };

    // Act
    let result = QueueItem::from_torrent(path, torrent);

    // Assert
    assert!(result.indexer.is_empty());
}

#[test]
fn from_torrent_with_missing_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = TorrentSummary {
        name: "Example Torrent".to_owned(),
        info_hash: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
        source: Some("ABC".to_owned()),
        comment: None,
        ..TorrentSummary::default()
    };

    // Act
    let result = QueueItem::from_torrent(path, torrent);

    // Assert
    assert!(result.id.is_none());
}

#[test]
fn from_torrent_with_invalid_comment() {
    // Arrange
    let path = PathBuf::from("/path/to/file.torrent");
    let torrent = TorrentSummary {
        name: "Example Torrent".to_owned(),
        info_hash: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
        source: Some("Indexer".to_owned()),
        comment: Some("invalid_url".to_owned()),
        ..TorrentSummary::default()
    };

    // Act
    let result = QueueItem::from_torrent(path, torrent);

    // Assert
    assert!(result.id.is_none());
}
