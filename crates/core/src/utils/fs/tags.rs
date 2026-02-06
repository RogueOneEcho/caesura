//! Read and manipulate audio file tags for transcoding.

use std::sync::LazyLock;

use crate::prelude::*;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::ItemKey::TrackNumber;
use lofty::tag::{Accessor, Tag, TagType};
use regex::Regex;

/// Match vinyl track numbering: letter followed by optional digits.
///
/// Example: `A1`, `B12`, `A`
static VINYL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Z])(\d+)?$").expect("regex should compile"));

/// Match `{track}/{total}` numbering format.
///
/// Example: `3/12`, `01/10`
static TOTAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)/(\d+)$").expect("regex should compile"));

/// Read Vorbis comment tags from a FLAC file.
pub(crate) fn get_vorbis_tags(flac: &FlacFile) -> Result<Tag, Error> {
    let file = Probe::open(flac.path.clone())
        .map_err(|e| error("get tags", e.to_string()))?
        .read()
        .map_err(|e| error("get tags", e.to_string()))?;
    if let Some(vorbis) = file.tag(TagType::VorbisComments) {
        Ok(vorbis.clone())
    } else {
        Err(error(
            "get tags",
            format!("No Vobis comments: {}", flac.path.display()),
        ))
    }
}

/// Re-map tags to `ID3v2` format for MP3 output.
pub(crate) fn convert_to_id3v2(tags: &mut Tag) {
    tags.re_map(TagType::Id3v2);
}

/// Ensure tags contain a numeric track number.
///
/// - Returns `true` if a numeric track number is already present or was successfully parsed
/// - Attempts `"track/total"` format first, then vinyl format
pub(crate) fn fix_track_numbering(tags: &mut Tag) -> bool {
    if tags.track().is_some() {
        return true;
    }
    if replace_total_track_numbering(tags).is_ok() {
        return true;
    }
    if replace_vinyl_track_numbering(tags).is_ok() {
        return true;
    }
    false
}

fn replace_vinyl_track_numbering(tags: &mut Tag) -> Result<(), Error> {
    let track = tags.get_string(&TrackNumber).ok_or_else(|| {
        error(
            "replace vinyl track numbering",
            "No track number string".to_owned(),
        )
    })?;
    let (disc_number, track_number) = get_numeric_from_vinyl_format(track).ok_or_else(|| {
        error(
            "replace vinyl track numbering",
            "Not vinyl format".to_owned(),
        )
    })?;
    trace!(
        "Replacing vinyl track ({track}) with numeric: track {track_number}, disc {disc_number}"
    );
    tags.set_disk(disc_number);
    tags.set_track(track_number);
    Ok(())
}

fn replace_total_track_numbering(tags: &mut Tag) -> Result<(), Error> {
    let track = tags.get_string(&TrackNumber).ok_or_else(|| {
        error(
            "replace total track numbering",
            "No track number string".to_owned(),
        )
    })?;
    let (track_number, track_total) = get_numeric_from_total_format(track).ok_or_else(|| {
        error(
            "replace total track numbering",
            "Not vinyl format".to_owned(),
        )
    })?;
    trace!(
        "Replacing total track numbering ({track}) with numeric: track {track_number}, total {track_total}"
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
pub(crate) fn get_numeric_from_total_format(input: &str) -> Option<(u32, u32)> {
    let captures = TOTAL_REGEX.captures(input)?;
    let track_number: u32 = captures.get(1)?.as_str().parse().ok()?;
    let track_total: u32 = captures.get(2)?.as_str().parse().ok()?;
    Some((track_number, track_total))
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
