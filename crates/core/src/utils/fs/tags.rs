//! Read and manipulate audio file tags for transcoding.

use crate::prelude::*;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::error::LoftyError;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::flac::FlacFile as LoftyFlacFile;
use lofty::id3::v2::{Frame, Id3v2Tag};
use lofty::prelude::TagExt;
use lofty::probe::Probe;
use lofty::tag::ItemKey::TrackNumber;
use lofty::tag::{Accessor, ItemKey, Tag, TagType};
use std::str::from_utf8;

/// Match vinyl track numbering: letter followed by optional digits.
///
/// Example: `A1`, `B12`, `A`
static VINYL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Z])(\d+)?$").expect("regex should compile"));

/// Match `{track}/{total}` numbering format.
///
/// Example: `3/12`, `01/10`, `3 / 12`
static SLASH_TOTAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\s*/\s*(\d+)$").expect("regex should compile"));

/// Match `{track} of {total}` numbering format.
///
/// Example: `1 of 10`, `01 of 12`, `1of10`
static OF_TOTAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\s*of\s*(\d+)$").expect("regex should compile"));

/// Read Vorbis comment tags from a FLAC file.
pub(crate) fn get_vorbis_tags(flac: &FlacFile) -> Result<Tag, Failure<TagsAction>> {
    let file = Probe::open(flac.path.clone())
        .map_err(Failure::wrap_with_path(TagsAction::OpenFile, &flac.path))?
        .read()
        .map_err(Failure::wrap_with_path(TagsAction::ReadTags, &flac.path))?;
    if let Some(vorbis) = file.tag(TagType::VorbisComments) {
        Ok(vorbis.clone())
    } else {
        Err(
            Failure::new(TagsAction::GetVorbisComments, TagsError::NoVorbisComments)
                .with_path(&flac.path),
        )
    }
}

/// Re-map tags to `ID3v2` format for MP3 output.
pub(crate) fn convert_to_id3v2(tags: &mut Tag) {
    tags.re_map(TagType::Id3v2);
}

/// Ensure tags contain a numeric track number.
///
/// - Returns `true` if a numeric track number is already present or was successfully parsed
/// - Attempts slash format, "of" format, then vinyl format
pub(crate) fn fix_track_numbering(tags: &mut Tag) -> bool {
    if tags.track().is_some() {
        return true;
    }
    if replace_slash_total_track_numbering(tags).is_ok() {
        return true;
    }
    if replace_of_total_track_numbering(tags).is_ok() {
        return true;
    }
    if replace_vinyl_track_numbering(tags).is_ok() {
        return true;
    }
    false
}

fn replace_vinyl_track_numbering(tags: &mut Tag) -> Result<(), Failure<TagsAction>> {
    let track = tags
        .get_string(TrackNumber)
        .ok_or_else(|| Failure::new(TagsAction::ReadTags, TagsError::NoTrackNumber))?;
    let (disc_number, track_number) = get_numeric_from_vinyl_format(track).ok_or_else(|| {
        Failure::new(TagsAction::ReadTags, TagsError::InvalidFormat).with("track", track)
    })?;
    trace!(
        "Replacing vinyl track ({track}) with numeric: track {track_number}, disc {disc_number}"
    );
    tags.set_disk(disc_number);
    tags.set_track(track_number);
    Ok(())
}

fn replace_slash_total_track_numbering(tags: &mut Tag) -> Result<(), Failure<TagsAction>> {
    let track = tags
        .get_string(TrackNumber)
        .ok_or_else(|| Failure::new(TagsAction::ReadTags, TagsError::NoTrackNumber))?;
    let (track_number, track_total) =
        get_numeric_from_slash_total_format(track).ok_or_else(|| {
            Failure::new(TagsAction::ReadTags, TagsError::InvalidFormat).with("track", track)
        })?;
    trace!(
        "Replacing total track numbering ({track}) with numeric: track {track_number}, total {track_total}"
    );
    tags.set_track(track_number);
    tags.set_track_total(track_total);
    Ok(())
}

fn replace_of_total_track_numbering(tags: &mut Tag) -> Result<(), Failure<TagsAction>> {
    let track = tags
        .get_string(TrackNumber)
        .ok_or_else(|| Failure::new(TagsAction::ReadTags, TagsError::NoTrackNumber))?;
    let (track_number, track_total) = get_numeric_from_of_total_format(track).ok_or_else(|| {
        Failure::new(TagsAction::ReadTags, TagsError::InvalidFormat).with("track", track)
    })?;
    trace!(
        "Replacing of-total track numbering ({track}) with numeric: track {track_number}, total {track_total}"
    );
    tags.set_track(track_number);
    tags.set_track_total(track_total);
    Ok(())
}

/// Parse vinyl-style track numbering into `(disc, track)`.
///
/// Examples:
/// - `A1` → `(1, 1)`
/// - `A2` → `(1, 2)`
/// - `B1` → `(2, 1)`
/// - `A`  → `(1, 1)`
pub(crate) fn get_numeric_from_vinyl_format(input: &str) -> Option<(u32, u32)> {
    let captures = VINYL_REGEX.captures(input)?;
    let disc_letter = captures.get(1)?.as_str().chars().next()?;
    let track_number: u32 = captures.get(2).map_or(Ok(1), |m| m.as_str().parse()).ok()?;
    let disc_number = letter_to_number(disc_letter)?;
    Some((disc_number, track_number))
}

/// Parse `"track/total"` format into `(track, total)`.
pub(crate) fn get_numeric_from_slash_total_format(input: &str) -> Option<(u32, u32)> {
    let captures = SLASH_TOTAL_REGEX.captures(input)?;
    let track_number: u32 = captures.get(1)?.as_str().parse().ok()?;
    let track_total: u32 = captures.get(2)?.as_str().parse().ok()?;
    Some((track_number, track_total))
}

/// Parse `"track of total"` format into `(track, total)`.
pub(crate) fn get_numeric_from_of_total_format(input: &str) -> Option<(u32, u32)> {
    let captures = OF_TOTAL_REGEX.captures(input)?;
    let track_number: u32 = captures.get(1)?.as_str().parse().ok()?;
    let track_total: u32 = captures.get(2)?.as_str().parse().ok()?;
    Some((track_number, track_total))
}

/// Remove tag items matching the given [`ItemKey`] list.
pub(crate) fn exclude_tags(tags: &mut Tag, keys: &[ItemKey]) {
    for key in keys {
        if let Some(value) = tags.get_string(*key) {
            trace!("Excluding {key:?}: {value}");
            tags.remove_key(*key);
        }
    }
}

/// Map Vorbis comment names to [`ItemKey`] values.
///
/// - Unrecognized names are logged as a warning and dropped
pub(crate) fn vorbis_keys(names: &[String]) -> Vec<ItemKey> {
    names
        .iter()
        .filter_map(|name| {
            let key = ItemKey::from_key(TagType::VorbisComments, name);
            if key.is_none() {
                warn!("Ignoring unknown Vorbis key: {name}");
            }
            key
        })
        .collect()
}

/// Exclude specified Vorbis comments from a FLAC file on disk.
///
/// - Uses the native [`VorbisComments`](lofty::ogg::VorbisComments) type
/// - Preserves all Vorbis comment keys including those without an [`ItemKey`] mapping
///   (e.g. `SYNCEDLYRICS`, `DISCOGS_*`)
/// - Only writes the file if at least one tag was actually removed
pub(crate) fn exclude_vorbis_comments_from_flac(
    path: &Path,
    keys: &[String],
) -> Result<(), Failure<TagsAction>> {
    if keys.is_empty() {
        return Ok(());
    }
    let mut file = File::open(path).map_err(Failure::wrap_with_path(TagsAction::OpenFile, path))?;
    let mut flac = LoftyFlacFile::read_from(&mut file, ParseOptions::default())
        .map_err(Failure::wrap_with_path(TagsAction::ReadTags, path))?;
    let Some(vorbis) = flac.vorbis_comments_mut() else {
        return Ok(());
    };
    let mut removed_any = false;
    for key in keys {
        if vorbis.remove(key).count() > 0 {
            removed_any = true;
        }
    }
    if !removed_any {
        return Ok(());
    }
    vorbis
        .save_to_path(path, WriteOptions::default())
        .map_err(Failure::wrap_with_path(TagsAction::WriteTags, path))?;
    Ok(())
}

/// Convert a generic [`Tag`] to [`Id3v2Tag`] and save with deterministic frame ordering.
///
/// lofty 0.23's `Tag` to `Id3v2Tag` conversion collects frames into `HashSet`/`HashMap`,
/// producing correct frames in non-deterministic order. Sorting by frame ID before writing
/// ensures stable binary output.
pub(crate) fn save_id3v2_deterministic(tags: Tag, path: &Path) -> Result<(), LoftyError> {
    let id3 = Id3v2Tag::from(tags);
    let mut frames: Vec<Frame<'static>> = id3.into_iter().collect();
    frames.sort_by_key(frame_sort_key);
    let mut sorted = Id3v2Tag::new();
    for frame in frames {
        sorted.insert(frame);
    }
    sorted.save_to_path(path, WriteOptions::default())
}

/// Deterministic sort key for an `ID3v2` frame.
///
/// Most frames are unique by ID. Multi-instance frames (TXXX, WXXX, COMM, USLT)
/// are disambiguated by description and language.
fn frame_sort_key(frame: &Frame<'_>) -> String {
    let id = frame.id_str();
    match frame {
        Frame::UserText(f) => format!("{id}\0{}", f.description),
        Frame::UserUrl(f) => format!("{id}\0{}", f.description),
        Frame::Comment(f) => {
            let lang = from_utf8(&f.language).unwrap_or_default();
            format!("{id}\0{lang}\0{}", f.description)
        }
        Frame::UnsynchronizedText(f) => {
            let lang = from_utf8(&f.language).unwrap_or_default();
            format!("{id}\0{lang}\0{}", f.description)
        }
        _ => String::from(id),
    }
}

/// Print all tag key-value pairs to stdout for debugging.
#[allow(dead_code)]
pub(crate) fn print_tags(tags: &Tag) {
    for item in tags.items() {
        let key = item.key();
        let value = item.value();
        println!("{key:?}: {value:?}");
    }
}

#[allow(clippy::as_conversions)]
fn letter_to_number(letter: char) -> Option<u32> {
    match letter {
        'A'..='Z' => Some((letter as u32) - ('A' as u32) + 1),
        _ => None,
    }
}
