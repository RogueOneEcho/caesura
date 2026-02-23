//! Builder for formatting data as aligned text tables.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// Number of spaces between columns.
const COLUMN_GAP: usize = 3;

/// Builder for creating aligned text tables.
///
/// - Columns are left-aligned by default, separated by [`COLUMN_GAP`] spaces
/// - Use [`right_align`](Self::right_align) to right-align specific columns
/// - Headers remain left-aligned even when data columns are right-aligned
pub(crate) struct TableBuilder<'a> {
    headers: Option<Vec<Vec<Cow<'a, str>>>>,
    rows: Vec<Vec<Cow<'a, str>>>,
    right_aligned: Vec<bool>,
    newline_after_headers: bool,
}

impl<'a> TableBuilder<'a> {
    /// Create a new table builder.
    pub(crate) fn new() -> Self {
        Self {
            headers: None,
            rows: Vec::new(),
            right_aligned: Vec::new(),
            newline_after_headers: false,
        }
    }

    /// Set single-line headers.
    ///
    /// Uses associated type bound instead of nested `impl Trait` for IDE compatibility.
    ///
    /// - [intellij-rust#8414](https://github.com/intellij-rust/intellij-rust/issues/8414)
    #[cfg(test)]
    fn headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.headers = Some(headers.into_iter().map(|h| vec![h.into()]).collect());
        self
    }

    /// Set multi-line headers.
    ///
    /// Each column header is a list of lines, rendered bottom-aligned so that
    /// shorter headers are padded with blank lines at the top.
    pub(crate) fn multi_line_headers<I, J>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = J>,
        J: IntoIterator,
        <J as IntoIterator>::Item: Into<Cow<'a, str>>,
    {
        self.headers = Some(
            headers
                .into_iter()
                .map(|col| col.into_iter().map(Into::into).collect())
                .collect(),
        );
        self
    }

    /// Set column alignments (true = right-aligned, false = left-aligned).
    ///
    /// Headers remain left-aligned; only data rows are affected.
    pub(crate) fn right_align(mut self, columns: Vec<bool>) -> Self {
        self.right_aligned = columns;
        self
    }

    /// Add a blank line between headers and data rows.
    pub(crate) fn newline_after_headers(mut self) -> Self {
        self.newline_after_headers = true;
        self
    }

    /// Add a data row.
    ///
    /// Uses associated type bound instead of nested `impl Trait` for IDE compatibility.
    ///
    /// - [intellij-rust#8414](https://github.com/intellij-rust/intellij-rust/issues/8414)
    pub(crate) fn row<I>(mut self, row: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    /// Build the formatted table string.
    pub(crate) fn build(self) -> String {
        let widths = self.column_widths();
        let no_right_align = Vec::new();
        let mut output = String::new();
        if let Some(headers) = &self.headers {
            let max_lines = headers.iter().map(Vec::len).max().unwrap_or(0);
            for line_idx in 0..max_lines {
                let row: Vec<Cow<'_, str>> = headers
                    .iter()
                    .map(|col| {
                        let offset = max_lines - col.len();
                        col.get(line_idx.wrapping_sub(offset))
                            .map_or(Cow::Borrowed(""), |s| Cow::Borrowed(s.as_ref()))
                    })
                    .collect();
                format_row(&mut output, &row, &widths, &no_right_align);
            }
            if self.newline_after_headers {
                output.push('\n');
            }
        }
        for row in &self.rows {
            format_row(&mut output, row, &widths, &self.right_aligned);
        }
        output
    }

    /// Calculate the maximum width of each column.
    fn column_widths(&self) -> Vec<usize> {
        let header_cols = self.headers.as_ref().map_or(0, Vec::len);
        let max_row_cols = self.rows.iter().map(Vec::len).max().unwrap_or(0);
        let col_count = header_cols.max(max_row_cols);
        let mut widths = vec![0; col_count];
        if let Some(headers) = &self.headers {
            for (width, col) in widths.iter_mut().zip(headers) {
                for line in col {
                    *width = (*width).max(visible_width(line));
                }
            }
        }
        for row in &self.rows {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(visible_width(cell));
            }
        }
        widths
    }
}

/// Visible width of a string in terminal columns.
///
/// - Strips ANSI SGR escape sequences before measuring
/// - Uses [`UnicodeWidthStr`] for accurate column widths (e.g., CJK characters
///   occupy 2 columns, emoji occupy 2 columns, zero-width characters occupy 0)
fn visible_width(s: &str) -> usize {
    static ANSI_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").expect("ANSI regex should be valid"));
    ANSI_RE.replace_all(s, "").width()
}

/// Format a single row with proper padding.
fn format_row(
    output: &mut String,
    cells: &[Cow<'_, str>],
    widths: &[usize],
    right_aligned: &[bool],
) {
    let Some((last, rest)) = cells.split_last() else {
        output.push('\n');
        return;
    };
    for (i, (cell, &width)) in rest.iter().zip(widths).enumerate() {
        let visible = visible_width(cell);
        let padding = width.saturating_sub(visible);
        if right_aligned.get(i).copied().unwrap_or(false) {
            output.push_str(&" ".repeat(padding));
            output.push_str(cell);
        } else {
            output.push_str(cell);
            output.push_str(&" ".repeat(padding));
        }
        output.push_str(&" ".repeat(COLUMN_GAP));
    }
    output.push_str(last);
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn table_with_headers_and_rows() {
        let table = TableBuilder::new()
            .headers(["Name", "Age"])
            .row(["Alice", "30"])
            .row(["Bob", "25"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_without_headers() {
        let table = TableBuilder::new().row(["A", "B"]).row(["C", "D"]).build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_varying_widths() {
        let table = TableBuilder::new()
            .row(["Short", "X"])
            .row(["A", "LongerValue"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn empty_table() {
        let table = TableBuilder::new().build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_mixed_owned_and_borrowed() {
        let owned = String::from("Owned");
        let table = TableBuilder::new()
            .headers(["Borrowed", &owned])
            .row([
                Cow::Borrowed("literal"),
                Cow::Owned(format!("computed {}", 42)),
            ])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_varying_column_counts() {
        let table = TableBuilder::new()
            .headers(["A", "B", "C"])
            .row(["1", "2", "3"])
            .row(["4", "5"])
            .row(["6"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_right_aligned_columns() {
        let table = TableBuilder::new()
            .headers(["Name", "Size", "Count"])
            .right_align(vec![false, true, true])
            .row(["foo.txt", "1.2 MB", "42"])
            .row(["bar.txt", "956 KB", "7"])
            .row(["baz.txt", "12.5 MB", "128"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn visible_width_plain_text() {
        assert_eq!(visible_width("hello"), 5);
    }

    #[test]
    fn visible_width_empty_string() {
        assert_eq!(visible_width(""), 0);
    }

    #[test]
    fn visible_width_single_ansi_code() {
        // \x1b[31m = red, \x1b[0m = reset
        assert_eq!(visible_width("\x1b[31mhello\x1b[0m"), 5);
    }

    #[test]
    fn visible_width_multiple_ansi_codes() {
        // bold + red + text + reset
        assert_eq!(visible_width("\x1b[1m\x1b[31merror\x1b[0m"), 5);
    }

    #[test]
    fn visible_width_ansi_with_semicolons() {
        // \x1b[1;31m = bold red (compound SGR)
        assert_eq!(visible_width("\x1b[1;31mwarning\x1b[0m"), 7);
    }

    #[test]
    fn visible_width_mixed_plain_and_ansi() {
        assert_eq!(visible_width("plain \x1b[2mdimmed\x1b[0m end"), 16);
    }

    #[test]
    fn visible_width_multibyte_utf8() {
        // ⚠ is 3 bytes but 1 character
        assert_eq!(visible_width("⚠"), 1);
    }

    #[test]
    fn visible_width_multibyte_utf8_with_ansi() {
        assert_eq!(visible_width("\x1b[31m⚠\x1b[0m"), 1);
    }

    #[test]
    fn visible_width_emoji() {
        // Most emoji occupy 2 terminal columns
        assert_eq!(visible_width("✅"), 2);
        assert_eq!(visible_width("🎵"), 2);
    }

    #[test]
    fn visible_width_emoji_with_ansi() {
        assert_eq!(visible_width("\x1b[33m🎵\x1b[0m"), 2);
    }

    #[test]
    fn table_with_multi_line_headers() {
        let table = TableBuilder::new()
            .multi_line_headers([
                vec!["D"],
                vec!["T"],
                vec!["Time"],
                vec!["Size"],
                vec!["Bit", "Rate", "kbps"],
                vec!["Sample", "Rate", "kHz"],
                vec!["Channels"],
                vec!["Bit", "Depth"],
            ])
            .row(["1", "1", "01:05", "830 KiB", "104", "44.1", "2", "16"])
            .row(["2", "1", "01:05", "936 KiB", "117", "44.1", "2", "16"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_ansi_colored_cells() {
        let table = TableBuilder::new()
            .row(["plain", "\x1b[31mred\x1b[0m", "after"])
            .row(["a", "b", "c"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_multibyte_and_ansi_cells() {
        let table = TableBuilder::new()
            .row(["name", "\x1b[31m⚠\x1b[0m", "detail"])
            .row(["longer", "ok", "other"])
            .build();
        assert_snapshot!(table);
    }
}
