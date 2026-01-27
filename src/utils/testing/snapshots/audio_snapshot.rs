use lofty::file::{AudioFile, TaggedFile, TaggedFileExt};
use lofty::picture::Picture;
use lofty::probe::Probe;
use lofty::properties::{ChannelMask, FileProperties};
use lofty::tag::{ItemValue, Tag};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// Snapshot of audio file metadata for deterministic testing.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AudioSnapshot {
    /// Duration in milliseconds.
    pub duration_ms: u128,
    /// Overall bitrate including all streams.
    pub overall_bitrate: Option<u32>,
    /// Audio-only bitrate.
    pub audio_bitrate: Option<u32>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Bit depth.
    pub bit_depth: Option<u8>,
    /// Number of audio channels.
    pub channels: Option<u8>,
    /// Channel mask bits.
    pub channel_mask: Option<u32>,
    /// Tags grouped by tag type (e.g., `VorbisComments`, `ID3v2`).
    pub tags: BTreeMap<String, BTreeMap<String, TagValue>>,
    /// Embedded pictures.
    pub pictures: Vec<PictureSnapshot>,
}

/// Snapshot of an embedded picture for deterministic testing.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PictureSnapshot {
    /// Picture type (e.g., `CoverFront`).
    pub picture_type: String,
    /// MIME type.
    pub mime_type: Option<String>,
    /// Picture description.
    pub description: Option<String>,
    /// Size in bytes.
    pub size: usize,
    /// SHA-256 hash of picture data.
    pub sha256: String,
}

impl From<&Picture> for PictureSnapshot {
    fn from(picture: &Picture) -> Self {
        Self {
            picture_type: format!("{:?}", picture.pic_type()),
            mime_type: picture.mime_type().map(|m| format!("{m:?}")),
            description: picture.description().map(String::from),
            size: picture.data().len(),
            sha256: format!("{:x}", Sha256::digest(picture.data())),
        }
    }
}

impl From<&FileProperties> for AudioSnapshot {
    fn from(props: &FileProperties) -> Self {
        Self {
            duration_ms: props.duration().as_millis(),
            overall_bitrate: props.overall_bitrate(),
            audio_bitrate: props.audio_bitrate(),
            sample_rate: props.sample_rate(),
            bit_depth: props.bit_depth(),
            channels: props.channels(),
            channel_mask: props.channel_mask().map(ChannelMask::bits),
            tags: BTreeMap::new(),
            pictures: Vec::new(),
        }
    }
}

impl From<&TaggedFile> for AudioSnapshot {
    fn from(file: &TaggedFile) -> Self {
        let mut metadata = Self::from(file.properties());
        metadata.tags = get_tag_map(file.tags());
        metadata.pictures = get_pictures(file.tags());
        metadata
    }
}

impl AudioSnapshot {
    /// Create an [`AudioSnapshot`] from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let file = Probe::open(path).ok()?.read().ok()?;
        Some(Self::from(&file))
    }
}

/// Tag value that can be text or binary data.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum TagValue {
    /// Text value.
    Text(String),
    /// Binary data represented by size and hash.
    Binary {
        /// Size in bytes.
        size: usize,
        /// SHA-256 hash.
        sha256: String,
    },
}

impl From<&ItemValue> for TagValue {
    fn from(value: &ItemValue) -> Self {
        match value {
            ItemValue::Text(text) | ItemValue::Locator(text) => Self::Text(text.clone()),
            ItemValue::Binary(data) => Self::Binary {
                size: data.len(),
                sha256: format!("{:x}", Sha256::digest(data)),
            },
        }
    }
}

/// Collect tags grouped by tag type (`ID3v1`, `ID3v2`, `VorbisComments`, etc.)
/// Each tag type can only appear once per file
fn get_tag_map(tags: &[Tag]) -> BTreeMap<String, BTreeMap<String, TagValue>> {
    let mut map: BTreeMap<String, BTreeMap<String, TagValue>> = BTreeMap::new();
    for tag in tags {
        let tag_type = format!("{:?}", tag.tag_type());
        let tag_items = map.entry(tag_type).or_default();

        for item in tag.items() {
            let key = format!("{:?}", item.key());
            tag_items.insert(key, TagValue::from(item.value()));
        }
    }
    map
}

/// Collect pictures from all tags, sorted for deterministic output
fn get_pictures(tags: &[Tag]) -> Vec<PictureSnapshot> {
    let mut pictures: Vec<PictureSnapshot> = tags
        .iter()
        .flat_map(Tag::pictures)
        .map(PictureSnapshot::from)
        .collect();
    pictures.sort();
    pictures
}
