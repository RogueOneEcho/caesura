use crate::errors::error;
use regex::Regex;
use rogue_logging::Error;

pub fn get_torrent_id_from_url(url: &str) -> Result<u32, Error> {
    get_torrent_id_from_group_url(url)
        .or_else(|| get_torrent_id_from_torrent_url(url))
        .ok_or_else(|| error("get torrent id from url", "failed to parse id".to_owned()))
}

#[must_use]
pub fn get_torrent_id_from_group_url(url: &str) -> Option<u32> {
    let id = Regex::new(r"/torrents\.php\?id=(\d+)&torrentid=(\d+)(#torrent\d+)?$")
        .expect("Regex should compile")
        .captures(url)?
        .get(2)?
        .as_str()
        .parse::<u32>()
        .expect("Number can be parsed");
    Some(id)
}

#[must_use]
pub fn get_torrent_id_from_torrent_url(url: &str) -> Option<u32> {
    let id = Regex::new(r"/torrents\.php\?torrentid=(\d+)(#torrent\d+)?$")
        .expect("Regex should compile")
        .captures(url)?
        .get(1)?
        .as_str()
        .parse::<u32>()
        .expect("Number can be parsed");
    Some(id)
}

#[must_use]
#[allow(dead_code)]
pub fn get_group_id_from_url(url: &str) -> Option<u32> {
    let id = Regex::new(r"/torrents\.php\?id=(\d+)&torrentid=(\d+)(#torrent\d+)?$")
        .expect("Regex should compile")
        .captures(url)?
        .get(1)?
        .as_str()
        .parse::<u32>()
        .expect("Number can be parsed");
    Some(id)
}

#[must_use]
pub fn get_permalink(base: &String, group_id: u32, torrent_id: u32) -> String {
    format!(r"{base}/torrents.php?id={group_id}&torrentid={torrent_id}#torrent{torrent_id}")
}
