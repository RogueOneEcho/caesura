//! Formatting utilities for inspect output.

use super::picture_info::PictureInfo;
use super::track_info::{TagEntry, TrackInfo};
use crate::prelude::*;
use lofty::tag::ItemKey;

pub(crate) const DIVIDER: &str = "\n---\n";

impl TrackInfo {
    /// Duration formatted as MM:SS.
    #[expect(
        clippy::integer_division,
        reason = "intentional integer division for MM:SS"
    )]
    fn format_duration(&self) -> String {
        let total_secs = self.duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{minutes:02}:{seconds:02}")
    }

    /// File size formatted as KB or MB.
    fn format_file_size(&self) -> String {
        format_size(self.file_size)
    }

    /// Format the audio properties table.
    pub(crate) fn format_properties_table(tracks: &[TrackInfo]) -> String {
        const L: bool = false;
        const R: bool = true;
        let has_flac_columns = tracks.iter().any(|t| t.bit_depth.is_some());
        let has_mixed_types = tracks
            .first()
            .is_some_and(|first| tracks.iter().any(|t| t.file_type != first.file_type));
        let mut headers: Vec<&str> = vec!["D", "T"];
        let mut align = vec![L, L];
        if has_mixed_types {
            headers.push("Type");
            align.push(L);
        }
        headers.extend(["Time", "Size", "Bitrate", "Sample Rate", "Channels"]);
        align.extend([R, R, R, L, L]);
        if has_flac_columns {
            headers.push("Bit Depth");
            align.push(L);
        }
        let mut builder = TableBuilder::new().headers(headers).right_align(align);
        for track in tracks {
            let mut row = vec![
                track.disc.clone().unwrap_or_default(),
                track.track.clone().unwrap_or_default(),
            ];
            if has_mixed_types {
                row.push(track.file_type.clone());
            }
            row.extend([
                track.format_duration(),
                track.format_file_size(),
                format!("{} kbps", track.bitrate),
                format!("{} Hz", track.sample_rate),
                track.channels.clone(),
            ]);
            if has_flac_columns {
                row.push(track.bit_depth.map_or_else(String::new, |d| d.to_string()));
            }
            builder = builder.row(row);
        }
        builder.build()
    }

    /// Format all per-track tags and pictures.
    pub(crate) fn format_all_tags(tracks: &[TrackInfo]) -> String {
        let mut output = String::new();
        for (i, track) in tracks.iter().enumerate() {
            if i > 0 {
                output.push_str(DIVIDER);
            }
            output.push_str(&track.format_tags());
        }
        output
    }

    /// Format tags and pictures as tables.
    fn format_tags(&self) -> String {
        let mut output = String::from("\n");
        output.push_str(&self.sub_path);
        output.push_str("\n\n");
        if !self.tags.is_empty() {
            output.push_str(&self.format_tags_table());
        }
        if !self.pictures.is_empty() {
            output.push('\n');
            output.push_str(&self.format_pictures_table());
        }
        output
    }

    /// Format tags as a table with NATIVE, ITEM, VALUE columns.
    fn format_tags_table(&self) -> String {
        let mut builder = TableBuilder::new();
        for entry in &self.tags {
            builder = builder.row([
                entry.native.as_deref().unwrap_or("").to_owned(),
                entry.format_item(),
                entry.value.clone(),
            ]);
        }
        builder.build()
    }

    /// Format pictures as a table with NATIVE, ITEM, SIZE, MIME columns.
    fn format_pictures_table(&self) -> String {
        let mut builder = TableBuilder::new();
        for pic in &self.pictures {
            builder = builder.row(pic.as_row());
        }
        builder.build()
    }
}

impl TagEntry {
    /// Format the item key as a human-readable name.
    fn format_item(&self) -> String {
        if let ItemKey::Unknown(raw) = &self.key {
            return raw.clone();
        }
        split_camel_case(&format!("{:?}", self.key))
    }
}

impl PictureInfo {
    /// Format as a table row.
    fn as_row(&self) -> [String; 4] {
        [
            self.native.clone(),
            self.type_name.clone(),
            format_size(u64::try_from(self.size).unwrap_or(u64::MAX)),
            self.mime.clone(),
        ]
    }
}

/// Format a byte size as KiB or MiB.
#[expect(
    clippy::integer_division,
    reason = "intentional integer division for file size"
)]
fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        let mib = bytes / MIB;
        let kib_remainder = (bytes % MIB) / KIB;
        let decimal = kib_remainder * 10 / 1024;
        format!("{mib}.{decimal} MiB")
    } else {
        let kib = bytes / KIB;
        if kib >= 100 {
            format!("{kib} KiB")
        } else {
            let decimal = (bytes % KIB) * 10 / KIB;
            format!("{kib}.{decimal} KiB")
        }
    }
}

/// Split a CamelCase string into space-separated words.
fn split_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}
