//! Builder for formatting data as aligned text tables.

use std::borrow::Cow;
use std::sync::LazyLock;

use TableStyle::*;
use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// Number of spaces between columns in plain-text output.
const COLUMN_GAP: usize = 3;

/// Output format for table rendering.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TableStyle {
    /// Space-separated columns for terminal display.
    #[default]
    Plain,
    /// GitHub-flavored markdown with pipe delimiters.
    ///
    /// - Multi-line headers are joined with `<br>`
    /// - Pipe characters and newlines in cell content are escaped
    /// - Right-aligned columns use `---:` separator syntax
    Markdown,
}

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
    style: TableStyle,
}

impl<'a> TableBuilder<'a> {
    /// Create a new table builder.
    pub(crate) fn new() -> Self {
        Self {
            headers: None,
            rows: Vec::new(),
            right_aligned: Vec::new(),
            newline_after_headers: false,
            style: TableStyle::default(),
        }
    }

    /// Use GitHub-flavored markdown output with pipe delimiters.
    pub(crate) fn markdown(mut self) -> Self {
        self.style = Markdown;
        self
    }

    /// Set single-line headers.
    ///
    /// Uses associated type bound instead of nested `impl Trait` for IDE compatibility.
    ///
    /// - [intellij-rust#8414](https://github.com/intellij-rust/intellij-rust/issues/8414)
    pub(crate) fn headers<I>(mut self, headers: I) -> Self
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
    pub(crate) fn multi_line_headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoIterator,
        <I::Item as IntoIterator>::Item: Into<Cow<'a, str>>,
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
        let header_rows = self.prepare_headers();
        let rows = self.prepare_rows();
        let widths = self.column_widths(header_rows.as_deref(), &rows);
        let mut output = String::new();
        if let Some(header_rows) = &header_rows {
            for row in header_rows {
                output.push_str(&self.format_row(row, &widths, true));
            }
            if self.newline_after_headers {
                output.push('\n');
            }
            if self.style == Markdown {
                output.push_str(&self.markdown_separator_row(&widths));
            }
        }
        for row in &rows {
            output.push_str(&self.format_row(row, &widths, false));
        }
        output
    }

    /// Flatten and escape headers for the target style.
    fn prepare_headers(&self) -> Option<Vec<Vec<Cow<'_, str>>>> {
        self.headers.as_deref().map(|h| match self.style {
            Plain => expand_header_rows(h),
            Markdown => self.join_markdown_headers(h),
        })
    }

    /// Join multi-line headers into a single row with `<br>` separators.
    fn join_markdown_headers(&self, headers: &[Vec<Cow<'_, str>>]) -> Vec<Vec<Cow<'_, str>>> {
        let row = headers
            .iter()
            .map(|col| {
                let text = col
                    .iter()
                    .map(|line| self.escape_cell(line))
                    .collect::<Vec<_>>()
                    .join("<br>");
                Cow::Owned(text)
            })
            .collect();
        vec![row]
    }

    /// Escape row cells for the target style.
    fn prepare_rows(&self) -> Vec<Vec<Cow<'_, str>>> {
        self.rows
            .iter()
            .map(|row| row.iter().map(|cell| self.escape_cell(cell)).collect())
            .collect()
    }

    /// Calculate column widths from pre-processed header rows and data rows.
    fn column_widths(
        &self,
        header_rows: Option<&[Vec<Cow<'_, str>>]>,
        rows: &[Vec<Cow<'_, str>>],
    ) -> Vec<usize> {
        let header_cols = header_rows.map_or(0, |hrs| hrs.first().map_or(0, Vec::len));
        let max_row_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
        let min_width = if self.style == Markdown { 3 } else { 0 };
        let col_count = header_cols.max(max_row_cols);
        let mut widths = vec![min_width; col_count];
        if let Some(hrs) = header_rows {
            for row in hrs {
                for (width, cell) in widths.iter_mut().zip(row) {
                    *width = (*width).max(visible_width(cell));
                }
            }
        }
        for row in rows {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(visible_width(cell));
            }
        }
        widths
    }

    /// Escape characters that would break markdown table structure.
    fn escape_cell<'c>(&self, cell: &'c str) -> Cow<'c, str> {
        if self.style == Markdown && (cell.contains('|') || cell.contains('\n')) {
            Cow::Owned(cell.replace('|', "\\|").replace('\n', "<br>"))
        } else {
            Cow::Borrowed(cell)
        }
    }

    /// Format a single row of pre-processed cells.
    fn format_row(&self, cells: &[Cow<'_, str>], widths: &[usize], is_header: bool) -> String {
        let mut output = String::new();
        if self.style == Markdown {
            output.push('|');
        }
        for i in 0..widths.len() {
            let cell = cells.get(i).map_or("", Cow::as_ref);
            let width = widths.get(i).copied().unwrap_or(0);
            let is_right = if is_header {
                false
            } else {
                self.right_aligned.get(i).copied().unwrap_or(false)
            };
            if self.style == Markdown {
                output.push(' ');
            }
            let cell_width = visible_width(cell);
            let padding = width.saturating_sub(cell_width);
            if is_right {
                output.push_str(&" ".repeat(padding));
                output.push_str(cell);
            } else {
                output.push_str(cell);
                output.push_str(&" ".repeat(padding));
            }
            if self.style == Markdown {
                output.push_str(" |");
            } else {
                output.push_str(&" ".repeat(COLUMN_GAP));
            }
        }
        output = output.trim_end().to_owned();
        output.push('\n');
        output
    }

    /// Render the markdown separator row.
    fn markdown_separator_row(&self, widths: &[usize]) -> String {
        let mut output = String::new();
        output.push('|');
        for (i, &width) in widths.iter().enumerate() {
            let right = self.right_aligned.get(i).copied().unwrap_or(false);
            output.push(' ');
            if right {
                output.push_str(&"-".repeat(width - 1));
                output.push_str(": |");
            } else {
                output.push_str(&"-".repeat(width));
                output.push_str(" |");
            }
        }
        output.push('\n');
        output
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

/// Expand multi-line headers into multiple bottom-aligned rows.
///
/// Shorter headers are padded with blank lines at the top so all columns
/// align to the bottom row.
fn expand_header_rows<'a>(headers: &'a [Vec<Cow<'_, str>>]) -> Vec<Vec<Cow<'a, str>>> {
    let max_lines = headers.iter().map(Vec::len).max().unwrap_or(0);
    (0..max_lines)
        .map(|line_idx| {
            headers
                .iter()
                .map(|col| {
                    let offset = max_lines - col.len();
                    col.get(line_idx.wrapping_sub(offset))
                        .map_or(Cow::Borrowed(""), |s| Cow::Borrowed(s.as_ref()))
                })
                .collect()
        })
        .collect()
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

    #[test]
    fn markdown_table_with_headers_and_rows() {
        let table = TableBuilder::new()
            .markdown()
            .headers(["Name", "Age", "City"])
            .row(["Alice", "30", "New York"])
            .row(["Bob", "25", "London"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn markdown_table_with_pipe_escaping() {
        let table = TableBuilder::new()
            .markdown()
            .headers(["Key", "Value"])
            .row(["a|b", "x|y|z"])
            .row(["normal", "also normal"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn markdown_table_with_right_alignment() {
        let table = TableBuilder::new()
            .markdown()
            .headers(["Name", "Size", "Count"])
            .right_align(vec![false, true, true])
            .row(["foo.txt", "1.2 MB", "42"])
            .row(["bar.txt", "956 KB", "7"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn markdown_table_with_multi_line_headers() {
        let table = TableBuilder::new()
            .markdown()
            .multi_line_headers([vec!["Name"], vec!["Bit", "Rate"], vec!["Sample", "Rate"]])
            .row(["Track 1", "320", "44.1"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn markdown_table_without_headers() {
        let table = TableBuilder::new()
            .markdown()
            .row(["A", "B"])
            .row(["C", "D"])
            .build();
        assert_snapshot!(table);
    }
}
