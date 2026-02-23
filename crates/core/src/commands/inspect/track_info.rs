use super::picture_info::{PictureInfo, collect_pictures};
use crate::prelude::*;
use lofty::config::ParseOptions;
use lofty::file::{AudioFile, TaggedFile, TaggedFileExt};
use lofty::flac::FlacFile;
use lofty::mpeg::{ChannelMode, MpegFile};
use lofty::tag::{ItemKey, ItemValue};
use std::fs::File;
use std::time::Duration;

/// FLAC file extension.
const FLAC: &str = "flac";

/// MP3 file extension.
const MP3: &str = "mp3";

/// Supported audio file extensions.
pub(super) const EXTENSIONS: &[&str] = &[FLAC, MP3];

/// A single tag entry.
pub(super) struct TagEntry {
    /// The item key.
    pub(super) key: ItemKey,
    /// Native key (e.g. "COMMENT", "ARTIST").
    pub(super) native: Option<String>,
    /// Tag value.
    pub(super) value: String,
}

/// Per-track metadata collected from an audio file.
pub(super) struct TrackInfo {
    /// Path relative to the inspected directory.
    pub(super) sub_path: String,
    /// File type (e.g. "FLAC", "MP3").
    pub(super) file_type: String,
    /// File size in bytes.
    pub(super) file_size: u64,
    /// Track number from tags.
    pub(super) track: Option<String>,
    /// Disc number from tags.
    pub(super) disc: Option<String>,
    /// Audio duration.
    pub(super) duration: Duration,
    /// Audio bit rate in kbps.
    pub(super) bit_rate: u32,
    /// Sample rate in Hz.
    pub(super) sample_rate: u32,
    /// Channel description.
    pub(super) channels: String,
    /// Bit depth (FLAC only).
    pub(super) bit_depth: Option<u8>,
    /// Tag entries.
    pub(super) tags: Vec<TagEntry>,
    /// Embedded pictures.
    pub(super) pictures: Vec<PictureInfo>,
}

impl TrackInfo {
    /// Create a mock FLAC 16-bit 44.1 kHz [`TrackInfo`] for testing.
    #[cfg(test)]
    pub(super) fn mock_flac() -> Self {
        Self {
            sub_path: String::new(),
            file_type: "FLAC".to_owned(),
            file_size: 1_048_576,
            track: Some("1".to_owned()),
            disc: Some("1".to_owned()),
            duration: Duration::from_secs(60),
            bit_rate: 800,
            sample_rate: 44100,
            channels: "2".to_owned(),
            bit_depth: Some(16),
            tags: Vec::new(),
            pictures: Vec::new(),
        }
    }

    /// Create a mock MP3 320 kbps 44.1 kHz [`TrackInfo`] for testing.
    #[cfg(test)]
    pub(super) fn mock_mp3() -> Self {
        Self {
            sub_path: String::new(),
            file_type: "MP3".to_owned(),
            file_size: 2_457_600,
            track: Some("1".to_owned()),
            disc: Some("1".to_owned()),
            duration: Duration::from_secs(60),
            bit_rate: 320,
            sample_rate: 44100,
            channels: "Joint stereo".to_owned(),
            bit_depth: None,
            tags: Vec::new(),
            pictures: Vec::new(),
        }
    }

    /// Read all audio files from a directory.
    pub(crate) fn read_dir(dir: &Path) -> Result<Vec<TrackInfo>, Failure<InspectAction>> {
        let mut paths = DirectoryReader::new()
            .with_extensions(EXTENSIONS.to_vec())
            .read(dir)
            .map_err(Failure::wrap_with_path(InspectAction::ReadDir, dir))?;
        paths.sort();
        let mut tracks: Vec<TrackInfo> = Vec::new();
        for file_path in &paths {
            tracks.push(TrackInfo::read(dir, file_path)?);
        }
        Ok(tracks)
    }

    /// Read metadata from a single audio file, dispatching on extension.
    pub(super) fn read(base: &Path, path: &Path) -> Result<Self, Failure<InspectAction>> {
        let mut file =
            File::open(path).map_err(Failure::wrap_with_path(InspectAction::OpenFile, path))?;
        let file_size = file
            .metadata()
            .map_err(Failure::wrap_with_path(InspectAction::OpenFile, path))?
            .len();
        let sub_path = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let options = ParseOptions::default();
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let mut info = match extension.as_str() {
            FLAC => Self::from_flac(&mut file, path, options),
            MP3 => Self::from_mpeg(&mut file, path, options),
            _ => Err(
                Failure::new(InspectAction::OpenFile, InspectError::UnsupportedExtension)
                    .with("extension", extension)
                    .with_path(path),
            ),
        }?;
        info.sub_path = sub_path;
        info.file_size = file_size;
        Ok(info)
    }

    /// Create a [`TrackInfo`] from a FLAC file.
    fn from_flac(
        file: &mut File,
        path: &Path,
        options: ParseOptions,
    ) -> Result<Self, Failure<InspectAction>> {
        let flac = FlacFile::read_from(file, options)
            .map_err(Failure::wrap_with_path(InspectAction::ReadFlacFile, path))?;
        let props = *flac.properties();
        let tagged = TaggedFile::from(flac);
        Ok(Self {
            sub_path: String::new(),
            file_type: "FLAC".to_owned(),
            file_size: 0,
            track: get_tag_string(&tagged, &ItemKey::TrackNumber),
            disc: get_tag_string(&tagged, &ItemKey::DiscNumber),
            duration: props.duration(),
            bit_rate: props.audio_bitrate(),
            sample_rate: props.sample_rate(),
            channels: props.channels().to_string(),
            bit_depth: Some(props.bit_depth()),
            tags: collect_tags(&tagged),
            pictures: collect_pictures(&tagged),
        })
    }

    /// Create a [`TrackInfo`] from an MPEG (MP3) file.
    fn from_mpeg(
        file: &mut File,
        path: &Path,
        options: ParseOptions,
    ) -> Result<Self, Failure<InspectAction>> {
        let mpeg = MpegFile::read_from(file, options)
            .map_err(Failure::wrap_with_path(InspectAction::ReadMpegFile, path))?;
        let props = *mpeg.properties();
        let tagged = TaggedFile::from(mpeg);
        Ok(Self {
            sub_path: String::new(),
            file_type: "MP3".to_owned(),
            file_size: 0,
            track: get_tag_string(&tagged, &ItemKey::TrackNumber),
            disc: get_tag_string(&tagged, &ItemKey::DiscNumber),
            duration: props.duration(),
            bit_rate: props.audio_bitrate(),
            sample_rate: props.sample_rate(),
            channels: format_channel_mode(*props.channel_mode()),
            bit_depth: None,
            tags: collect_tags(&tagged),
            pictures: collect_pictures(&tagged),
        })
    }
}

/// Collect tag entries from a tagged file.
fn collect_tags(file: &TaggedFile) -> Vec<TagEntry> {
    let mut result = Vec::new();
    for tag in file.tags() {
        let tag_type = tag.tag_type();
        for item in tag.items() {
            let native = item.key().map_key(tag_type, true).map(ToOwned::to_owned);
            if let ItemValue::Text(text) | ItemValue::Locator(text) = item.value() {
                result.push(TagEntry {
                    key: item.key().clone(),
                    native,
                    value: text.clone(),
                });
            }
        }
    }
    result
}

/// Get a tag value as a string across all tags in a file.
fn get_tag_string(file: &TaggedFile, key: &ItemKey) -> Option<String> {
    file.tags()
        .iter()
        .find_map(|t| t.get_string(key))
        .map(ToOwned::to_owned)
}

/// Format a channel mode for display.
fn format_channel_mode(mode: ChannelMode) -> String {
    match mode {
        ChannelMode::Stereo => "Stereo".to_owned(),
        ChannelMode::JointStereo => "Joint stereo".to_owned(),
        ChannelMode::DualChannel => "Dual channel".to_owned(),
        ChannelMode::SingleChannel => "Mono".to_owned(),
    }
}
