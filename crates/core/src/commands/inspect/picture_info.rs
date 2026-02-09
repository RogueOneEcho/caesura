//! Embedded picture metadata collection.

use lofty::file::TaggedFile;
use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::tag::TagType;

/// Metadata about an embedded picture.
pub(super) struct PictureInfo {
    /// Native key for the picture storage (e.g. "APIC", "`METADATA_BLOCK_PICTURE`").
    pub(super) native: String,
    /// Picture type name (e.g. "Front Cover").
    pub(super) type_name: String,
    /// MIME type string.
    pub(super) mime: String,
    /// Size in bytes.
    pub(super) size: usize,
}

/// Collect pictures from a tagged file's tags.
pub(super) fn collect_pictures(file: &TaggedFile) -> Vec<PictureInfo> {
    let mut pictures = Vec::new();
    for tag in file.tags() {
        let native = picture_native_key(tag.tag_type());
        for pic in tag.pictures() {
            pictures.push(PictureInfo {
                native: native.to_owned(),
                type_name: format_picture_type(pic.pic_type()),
                mime: pic
                    .mime_type()
                    .map_or_else(|| "unknown".to_owned(), ToString::to_string),
                size: pic.data().len(),
            });
        }
    }
    pictures
}

/// Get the native key name for pictures in a given tag type.
fn picture_native_key(tag_type: TagType) -> &'static str {
    match tag_type {
        TagType::Id3v2 => "APIC",
        TagType::VorbisComments => "METADATA_BLOCK_PICTURE",
        TagType::Ape => "Cover Art",
        TagType::Mp4Ilst => "covr",
        _ => "PICTURE",
    }
}

/// Format a picture type for display.
fn format_picture_type(pic_type: PictureType) -> String {
    match pic_type {
        PictureType::CoverFront => "Front Cover".to_owned(),
        PictureType::CoverBack => "Back Cover".to_owned(),
        PictureType::Leaflet => "Leaflet".to_owned(),
        PictureType::Media => "Media".to_owned(),
        PictureType::LeadArtist => "Lead Artist".to_owned(),
        PictureType::Artist => "Artist".to_owned(),
        PictureType::Conductor => "Conductor".to_owned(),
        PictureType::Band => "Band".to_owned(),
        PictureType::Composer => "Composer".to_owned(),
        PictureType::Lyricist => "Lyricist".to_owned(),
        PictureType::RecordingLocation => "Recording Location".to_owned(),
        PictureType::DuringRecording => "During Recording".to_owned(),
        PictureType::DuringPerformance => "During Performance".to_owned(),
        PictureType::ScreenCapture => "Screen Capture".to_owned(),
        PictureType::BrightFish => "Bright Fish".to_owned(),
        PictureType::Illustration => "Illustration".to_owned(),
        PictureType::BandLogo => "Band Logo".to_owned(),
        PictureType::PublisherLogo => "Publisher Logo".to_owned(),
        PictureType::Icon => "Icon".to_owned(),
        PictureType::OtherIcon => "Other Icon".to_owned(),
        _ => "Other".to_owned(),
    }
}
