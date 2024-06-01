use crate::errors::AppError;
use regex::Regex;

pub fn get_torrent_id_from_url(url: &str, base: &String) -> Result<i64, AppError> {
    get_torrent_id_from_group_url(url, base)
        .or_else(|| get_torrent_id_from_torrent_url(url, base))
        .ok_or_else(|| {
            AppError::else_explained("get torrent id from url", "failed to parse id".to_owned())
        })
}

#[must_use]
pub fn get_torrent_id_from_group_url(url: &str, base: &String) -> Option<i64> {
    let id = Regex::new(
        format!(r"^{base}/torrents\.php\?id=(\d+)&torrentid=(\d+)(#torrent\d+)?$").as_str(),
    )
    .expect("Regex should compile")
    .captures(url)?
    .get(2)?
    .as_str()
    .parse::<i64>()
    .expect("Number can be parsed");
    Some(id)
}

#[must_use]
pub fn get_torrent_id_from_torrent_url(url: &str, base: &String) -> Option<i64> {
    let id = Regex::new(format!(r"^{base}/torrents\.php\?torrentid=(\d+)(#torrent\d+)?$").as_str())
        .expect("Regex should compile")
        .captures(url)?
        .get(1)?
        .as_str()
        .parse::<i64>()
        .expect("Number can be parsed");
    Some(id)
}

#[must_use]
pub fn get_group_id_from_url(url: &str, base: &String) -> Option<i64> {
    let id = Regex::new(
        format!(r"^{base}/torrents\.php\?id=(\d+)&torrentid=(\d+)(#torrent\d+)?$").as_str(),
    )
    .expect("Regex should compile")
    .captures(url)?
    .get(1)?
    .as_str()
    .parse::<i64>()
    .expect("Number can be parsed");
    Some(id)
}
