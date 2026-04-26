//! Builder for formatting data as aligned text tables.

use crate::prelude::*;
use TableStyle::*;
use std::borrow::Cow;

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
    max_column_widths: Vec<Option<usize>>,
    max_cell_lines: Option<usize>,
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
            max_column_widths: Vec::new(),
            max_cell_lines: None,
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

    /// Wrap cells in column `col` whose content exceeds `n` visible columns at
    /// word boundaries.
    ///
    /// - Continuation lines occupy additional visual rows aligned with the
    ///   column start
    /// - No effect in markdown style
    pub(crate) fn max_column_width(mut self, col: usize, n: usize) -> Self {
        if self.max_column_widths.len() <= col {
            self.max_column_widths.resize(col + 1, None);
        }
        if let Some(slot) = self.max_column_widths.get_mut(col) {
            *slot = Some(n);
        }
        self
    }

    /// Cap each cell at `n` visual rows.
    ///
    /// - Excess content is replaced with `[+X more lines, Y chars clipped]`
    ///   on the final retained row, which counts toward `n`
    /// - No effect in markdown style
    pub(crate) fn max_cell_lines(mut self, n: usize) -> Self {
        self.max_cell_lines = Some(n);
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
        let expanded_rows = self.expand_rows();
        let widths = self.column_widths(header_rows.as_deref(), &expanded_rows);
        let mut output = String::new();
        if let Some(header_rows) = &header_rows {
            for row in header_rows {
                let cells: Vec<&str> = row.iter().map(Cow::as_ref).collect();
                output.push_str(&self.format_row(&cells, &widths, true));
            }
            if self.newline_after_headers {
                output.push('\n');
            }
            if self.style == Markdown {
                output.push_str(&self.markdown_separator_row(&widths));
            }
        }
        for row in &expanded_rows {
            output.push_str(&self.format_expanded_row(row, &widths));
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

    /// Expand all data rows into per-cell visual lines, applying overflow
    /// options.
    ///
    /// - In markdown style, each cell is escaped and returned as a single
    ///   visual line; overflow options are ignored
    /// - In plain style, cells are escaped (no-op for plain) and passed through
    ///   [`expand_cell`] using the column's `max_column_width` and the global
    ///   `max_cell_lines`
    fn expand_rows(&self) -> Vec<Vec<Vec<String>>> {
        self.rows
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(col, cell)| {
                        let escaped = self.escape_cell(cell).into_owned();
                        if self.style != Plain {
                            return vec![escaped];
                        }
                        let max_width = self.max_column_widths.get(col).copied().flatten();
                        expand_cell(&escaped, max_width, self.max_cell_lines)
                    })
                    .collect()
            })
            .collect()
    }

    /// Calculate column widths from pre-processed header rows and expanded
    /// data rows.
    fn column_widths(
        &self,
        header_rows: Option<&[Vec<Cow<'_, str>>]>,
        expanded_rows: &[Vec<Vec<String>>],
    ) -> Vec<usize> {
        let header_cols = header_rows.map_or(0, |hrs| hrs.first().map_or(0, Vec::len));
        let max_row_cols = expanded_rows.iter().map(Vec::len).max().unwrap_or(0);
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
        for row in expanded_rows {
            for (col_idx, (width, cell_lines)) in widths.iter_mut().zip(row).enumerate() {
                for line in cell_lines {
                    let mut line_width = visible_width(line);
                    if self.style == Plain
                        && let Some(max) = self.max_column_widths.get(col_idx).copied().flatten()
                    {
                        line_width = line_width.min(max);
                    }
                    *width = (*width).max(line_width);
                }
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
    fn format_row(&self, cells: &[&str], widths: &[usize], is_header: bool) -> String {
        let mut output = String::new();
        if self.style == Markdown {
            output.push('|');
        }
        for i in 0..widths.len() {
            let cell = cells.get(i).copied().unwrap_or("");
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

    /// Format an expanded data row into one or more visual lines.
    ///
    /// Cells with fewer visual lines than the row height render their lines on
    /// the leading visual rows and blank padding on trailing rows.
    fn format_expanded_row(&self, cells: &[Vec<String>], widths: &[usize]) -> String {
        let height = cells.iter().map(Vec::len).max().unwrap_or(1);
        let mut output = String::new();
        for line_idx in 0..height {
            let row: Vec<&str> = cells
                .iter()
                .map(|c| c.get(line_idx).map_or("", String::as_str))
                .collect();
            output.push_str(&self.format_row(&row, widths, false));
        }
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

/// Expand a cell into a list of visual lines.
///
/// - Splits on `\n` first
/// - Word-wraps each source line if `max_width` is set
/// - Caps the resulting visual line count at `max_lines`, replacing the excess
///   with `[+X more lines, Y chars clipped]` on the final retained row
fn expand_cell(cell: &str, max_width: Option<usize>, max_lines: Option<usize>) -> Vec<String> {
    let source_lines: Vec<&str> = cell.split('\n').collect();
    let mut visual: Vec<String> = Vec::new();
    for src_line in &source_lines {
        let wrapped = match max_width {
            Some(w) => wrap_line(src_line, w),
            None => vec![(*src_line).to_owned()],
        };
        visual.extend(wrapped);
    }
    let Some(cap) = max_lines else {
        return visual;
    };
    if visual.len() <= cap {
        return visual;
    }
    if cap == 0 {
        return Vec::new();
    }
    let kept_visual = cap - 1;
    let dropped_visual_count = visual.len() - kept_visual;
    let dropped_chars = visual
        .get(kept_visual..)
        .map_or(0, |d| d.join("\n").chars().count());
    let mut output = visual.into_iter().take(kept_visual).collect::<Vec<_>>();
    output.push(
        format!("[+{dropped_visual_count} more lines, {dropped_chars} chars clipped]")
            .dimmed()
            .to_string(),
    );
    output
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
    fn expand_cell_no_constraints_splits_on_newline() {
        let lines = expand_cell("line one\nline two", None, None);
        assert_eq!(lines, vec!["line one", "line two"]);
    }

    #[test]
    fn expand_cell_wraps_long_line() {
        let lines = expand_cell("the quick brown fox jumps", Some(10), None);
        assert_eq!(lines, vec!["the quick", "brown fox", "jumps"]);
    }

    #[test]
    fn expand_cell_caps_lines_with_clip_note() {
        let input = "a\nb\nc\nd\ne";
        let lines = expand_cell(input, None, Some(3));
        assert_eq!(
            lines,
            vec![
                "a".to_owned(),
                "b".to_owned(),
                "[+3 more lines, 5 chars clipped]".dimmed().to_string(),
            ],
        );
    }

    #[test]
    fn expand_cell_clip_note_counts_after_last_retained_break() {
        let input = "alpha bravo charlie delta echo foxtrot";
        let lines = expand_cell(input, Some(11), Some(2));
        let mut iter = lines.iter();
        assert_eq!(iter.next().map(String::as_str), Some("alpha bravo"));
        let second = iter.next().expect("expected at least 2 lines");
        let plain = strip_ansi(second);
        assert!(
            plain.starts_with("[+") && plain.ends_with("chars clipped]"),
            "unexpected clip note: {second}",
        );
        assert!(iter.next().is_none(), "expected exactly 2 lines");
    }

    #[test]
    fn expand_cell_within_limit_no_clip_note() {
        let lines = expand_cell("a\nb", None, Some(3));
        assert_eq!(lines, vec!["a", "b"]);
    }

    #[test]
    fn table_with_max_column_width() {
        let table = TableBuilder::new()
            .max_column_width(1, 20)
            .row([
                "Item",
                "the quick brown fox jumps over the lazy dog",
                "Native",
            ])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_column_width_continuation() {
        let table = TableBuilder::new()
            .max_column_width(1, 15)
            .row(["A", "one two three four five six", "Z"])
            .row(["B", "short", "Y"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_column_width_other_columns_unaffected() {
        let table = TableBuilder::new()
            .max_column_width(1, 10)
            .row(["this column is not limited", "the quick brown fox", "tag"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_column_width_word_longer_than_limit() {
        let table = TableBuilder::new()
            .max_column_width(0, 10)
            .row(["https://example.com/very/long/path", "next"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_column_width_and_ansi() {
        let table = TableBuilder::new()
            .max_column_width(0, 11)
            .row(["\x1b[31mhello\x1b[0m world \x1b[31mhello\x1b[0m", "x"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_cell_lines_overflow() {
        let table = TableBuilder::new()
            .max_cell_lines(3)
            .row(["Lyrics", "line1\nline2\nline3\nline4\nline5", "X"])
            .build();
        let table = strip_ansi(&table);
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_cell_lines_within_limit() {
        let table = TableBuilder::new()
            .max_cell_lines(3)
            .row(["Tag", "line1\nline2", "X"])
            .build();
        assert_snapshot!(table);
    }

    #[test]
    fn table_with_max_column_width_and_max_cell_lines() {
        let table = TableBuilder::new()
            .max_column_width(1, 12)
            .max_cell_lines(3)
            .row([
                "Lyrics",
                "alpha bravo charlie delta echo foxtrot golf hotel",
                "X",
            ])
            .build();
        let table = strip_ansi(&table);
        assert_snapshot!(table);
    }

    #[test]
    fn table_builder_accepts_overflow_options() {
        let table = TableBuilder::new()
            .max_column_width(0, 10)
            .max_cell_lines(3)
            .row(["short", "x"])
            .build();
        assert_snapshot!(table);
    }

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
    fn markdown_table_with_max_column_width_ignored() {
        let table = TableBuilder::new()
            .markdown()
            .max_column_width(1, 10)
            .max_cell_lines(2)
            .headers(["Item", "Value"])
            .row(["copyright", "the quick brown fox jumps over the lazy dog"])
            .row(["lyrics", "line1\nline2\nline3\nline4"])
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
