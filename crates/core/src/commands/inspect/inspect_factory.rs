use super::picture_info::PictureInfo;
use super::track_info::{TagEntry, TrackInfo};
use crate::prelude::*;

/// Max width of the tag value column.
const MAX_VALUE_WIDTH: usize = 46;
/// Max number of lines for a tag value.
const MAX_VALUE_LINES: usize = 3;

/// Format inspect output with optional terminal styling.
pub struct InspectFactory {
    /// Apply ANSI color codes to output.
    style: bool,
}

impl InspectFactory {
    /// Create a new [`InspectFactory`].
    pub fn new(style: bool) -> Self {
        Self { style }
    }

    /// Get track details for a directory of audio files.
    ///
    /// Reads audio properties and tags natively using `lofty`.
    /// Auto-detects file format by extension (FLAC and MP3).
    pub(crate) fn create(&self, dir: &Path) -> Result<String, Failure<InspectAction>> {
        let (properties, tags) = self.create_split(dir)?;
        Ok(format!("{properties}{}{tags}", self.divider()))
    }

    /// Get track details split into properties table and per-track tags.
    pub(crate) fn create_split(
        &self,
        dir: &Path,
    ) -> Result<(String, String), Failure<InspectAction>> {
        let tracks = TrackInfo::read_dir(dir)?;
        let properties = self.format_properties_table(&tracks);
        let tags = self.format_all_tags(&tracks);
        Ok((properties, tags))
    }

    /// Format the audio properties table.
    pub(crate) fn format_properties_table(&self, tracks: &[TrackInfo]) -> String {
        const L: bool = false;
        const R: bool = true;
        let has_flac_columns = tracks.iter().any(|t| t.bit_depth.is_some());
        let has_mixed_types = tracks
            .first()
            .is_some_and(|first| tracks.iter().any(|t| t.file_type != first.file_type));
        let mut headers: Vec<Vec<&str>> = vec![vec!["D"], vec!["T"]];
        let mut align = vec![L, L];
        if has_mixed_types {
            headers.push(vec!["Type"]);
            align.push(L);
        }
        let kbps = self.style_unit("kbps");
        let khz = self.style_unit("kHz");
        headers.extend([
            vec!["Time"],
            vec!["Size"],
            vec!["Bit", "Rate", &kbps],
            vec!["Sample", "Rate", &khz],
            vec!["Channels"],
        ]);
        align.extend([R, R, R, L, L]);
        if has_flac_columns {
            headers.push(vec!["Bit", "Depth"]);
            align.push(L);
        }
        let headers = self.style_headers(headers);
        let mut builder = TableBuilder::new()
            .ansi(self.style)
            .multi_line_headers(headers)
            .right_align(align)
            .newline_after_headers();
        for track in tracks {
            let mut row = vec![
                self.style_key(track.disc.clone().unwrap_or_default()),
                self.style_key(track.track.clone().unwrap_or_default()),
            ];
            if has_mixed_types {
                row.push(track.file_type.clone());
            }
            row.extend([
                track.format_duration(),
                self.format_size(track.file_size),
                track.bit_rate.to_string(),
                track.format_sample_rate(),
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
    pub(crate) fn format_all_tags(&self, tracks: &[TrackInfo]) -> String {
        let mut output = String::new();
        for (i, track) in tracks.iter().enumerate() {
            if i > 0 {
                output.push_str(&self.divider());
            }
            output.push_str(&track.format_tags(self));
        }
        output
    }

    /// Format a byte size as KiB or MiB.
    #[expect(
        clippy::integer_division,
        reason = "intentional integer division for file size"
    )]
    fn format_size(&self, bytes: u64) -> String {
        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * 1024;
        if bytes >= MIB {
            let mib = bytes / MIB;
            let kib_remainder = (bytes % MIB) / KIB;
            let decimal = kib_remainder * 10 / 1024;
            format!("{mib}.{decimal} {}", self.style_unit("MiB"))
        } else {
            let kib = bytes / KIB;
            if kib >= 100 {
                format!("{kib} {}", self.style_unit("KiB"))
            } else {
                let decimal = (bytes % KIB) * 10 / KIB;
                format!("{kib}.{decimal} {}", self.style_unit("KiB"))
            }
        }
    }

    /// Horizontal rule divider between sections.
    pub(crate) fn divider(&self) -> String {
        if !self.style {
            return "\n---\n".to_owned();
        }
        "\n---\n".black().to_string()
    }

    fn style_headers(&self, headers: Vec<Vec<&str>>) -> Vec<Vec<String>> {
        if !self.style {
            return headers
                .into_iter()
                .map(|col| col.into_iter().map(ToString::to_string).collect())
                .collect();
        }
        headers
            .into_iter()
            .map(|col| col.into_iter().map(|a| self.style_info(a)).collect())
            .collect()
    }

    fn style_key(&self, input: impl Into<String>) -> String {
        if !self.style {
            return input.into();
        }
        input.into().yellow().to_string()
    }

    fn style_path(&self, input: impl Into<String>) -> String {
        if !self.style {
            return input.into();
        }
        input.into().green().to_string()
    }

    fn style_unit(&self, input: impl Into<String>) -> String {
        if !self.style {
            return input.into();
        }
        input.into().white().dimmed().to_string()
    }

    fn style_info(&self, input: impl Into<String>) -> String {
        if !self.style {
            return input.into();
        }
        input.into().cyan().to_string()
    }
}

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

    /// Sample rate formatted as kHz.
    #[expect(
        clippy::integer_division,
        reason = "intentional integer division for kHz conversion"
    )]
    fn format_sample_rate(&self) -> String {
        let khz = self.sample_rate / 1000;
        let remainder = self.sample_rate % 1000;
        if remainder == 0 {
            khz.to_string()
        } else {
            let decimal = remainder / 100;
            format!("{khz}.{decimal}")
        }
    }

    /// Format tags and pictures as tables.
    fn format_tags(&self, factory: &InspectFactory) -> String {
        let mut output = String::from("\n");
        output.push_str(&factory.style_path(&self.sub_path));
        output.push_str("\n\n");
        if !self.tags.is_empty() {
            output.push_str(&self.format_tags_table(factory));
        }
        if !self.pictures.is_empty() {
            output.push('\n');
            output.push_str(&self.format_pictures_table(factory));
        }
        output
    }

    /// Format tags as a table with NATIVE, ITEM, VALUE columns.
    fn format_tags_table(&self, factory: &InspectFactory) -> String {
        let mut builder = TableBuilder::new()
            .ansi(factory.style)
            .max_column_width(1, MAX_VALUE_WIDTH)
            .max_cell_lines(MAX_VALUE_LINES);
        for entry in &self.tags {
            builder = builder.row([
                factory.style_key(entry.format_item()),
                entry.value.clone(),
                factory.style_info(entry.native.as_deref().unwrap_or("")),
            ]);
        }
        builder.build()
    }

    /// Format pictures as a table with NATIVE, ITEM, SIZE, MIME columns.
    fn format_pictures_table(&self, factory: &InspectFactory) -> String {
        let mut builder = TableBuilder::new().ansi(factory.style);
        for pic in &self.pictures {
            builder = builder.row(pic.as_row(factory));
        }
        builder.build()
    }
}

impl TagEntry {
    /// Format the item key as a human-readable name.
    fn format_item(&self) -> String {
        split_camel_case(&format!("{:?}", self.key))
    }
}

impl PictureInfo {
    /// Format as a table row.
    fn as_row(&self, factory: &InspectFactory) -> [String; 4] {
        [
            factory.style_key(self.type_name.clone()),
            factory.format_size(u64::try_from(self.size).unwrap_or(u64::MAX)),
            self.mime.clone(),
            factory.style_info(self.native.clone()),
        ]
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
