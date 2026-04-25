use crate::testing_prelude::*;

#[test]
fn get_torrent_id_from_url_with_group() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992";
    assert_eq!(get_group_id_from_url(url), Some(2_259_978));
    assert_eq!(get_torrent_id_from_group_url(url), Some(4_871_992));
    assert_eq!(get_torrent_id_from_torrent_url(url), None);
    assert!(matches!(get_torrent_id_from_url(url), Ok(4_871_992)));
}

#[test]
fn get_torrent_id_from_url_without_group() {
    let url = "https://example.com/torrents.php?torrentid=4871992";
    assert_eq!(get_group_id_from_url(url), None);
    assert_eq!(get_torrent_id_from_group_url(url), None);
    assert_eq!(get_torrent_id_from_torrent_url(url), Some(4_871_992));
    assert!(matches!(get_torrent_id_from_url(url), Ok(4_871_992)));
}

#[test]
fn get_torrent_id_from_url_with_hash() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992";
    assert_eq!(get_group_id_from_url(url), Some(2_259_978));
    assert_eq!(get_torrent_id_from_group_url(url), Some(4_871_992));
    assert_eq!(get_torrent_id_from_torrent_url(url), None);
    assert!(matches!(get_torrent_id_from_url(url), Ok(4_871_992)));
}

#[test]
fn get_torrent_id_from_url_invalid() {
    let url = "https://example.com/torrents.php?abc";
    assert_eq!(get_group_id_from_url(url), None);
    assert_eq!(get_torrent_id_from_group_url(url), None);
    assert_eq!(get_torrent_id_from_torrent_url(url), None);
    assert!(get_torrent_id_from_url(url).is_err());
}

#[test]
fn get_torrent_id_from_url_incorrect_hash() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent1234567";
    assert_eq!(get_group_id_from_url(url), Some(2_259_978));
    assert_eq!(get_torrent_id_from_group_url(url), Some(4_871_992));
    assert_eq!(get_torrent_id_from_torrent_url(url), None);
    assert!(matches!(get_torrent_id_from_url(url), Ok(4_871_992)));
}

#[test]
fn get_report_url_with_torrent_id() {
    // Act
    let url = get_report_url("https://example.com", 4_871_992);
    // Assert
    assert_eq!(
        url,
        "https://example.com/reportsv2.php?action=report&id=4871992"
    );
}
