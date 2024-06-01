use crate::api::MusicInfo;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub wiki_body: String,
    pub bb_body: Option<String>,
    pub wiki_image: String,
    pub id: i64,
    pub name: String,
    pub year: i64,
    pub record_label: String,
    pub catalogue_number: String,
    pub release_type: i64,
    pub category_id: i64,
    pub category_name: String,
    pub time: String,
    pub vanity_house: bool,
    pub is_bookmarked: bool,
    pub tags: Vec<String>,
    pub music_info: MusicInfo,
}
