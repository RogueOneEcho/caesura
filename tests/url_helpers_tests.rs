use rogue_oxide::source::*;

const BASE_URL: &str = "https://example.com";

#[test]
fn get_group_id_from_url_without_hash() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992";
    let result = get_group_id_from_url(url, &BASE_URL.to_owned()).expect("Should be ok");
    assert_eq!(result, 2_259_978);
}

#[test]
fn get_group_id_from_url_with_hash() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992";
    let result = get_group_id_from_url(url, &BASE_URL.to_owned()).expect("Should be ok");
    assert_eq!(result, 2_259_978);
}

#[test]
fn get_torrent_id_from_url_group_and_torrent() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992";
    let result = get_torrent_id_from_url(url, &BASE_URL.to_owned()).expect("Should be ok");
    assert_eq!(result, 4_871_992);
}

#[test]
fn get_torrent_id_from_url_torrent_only() {
    let url = "https://example.com/torrents.php?torrentid=4871992";
    let result = get_torrent_id_from_url(url, &BASE_URL.to_owned()).expect("Should be ok");
    assert_eq!(result, 4_871_992);
}

#[test]
fn get_torrent_id_from_url_test_with_hash() {
    let url = "https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992";
    let result = get_torrent_id_from_url(url, &BASE_URL.to_owned()).expect("Should be ok");
    assert_eq!(result, 4_871_992);
}
