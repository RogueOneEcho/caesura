//! Builder for formatting data as aligned text tables.

use std::borrow::Cow;
use std::fmt::Write;

/// Number of spaces between columns.
const COLUMN_GAP: usize = 3;

/// Builder for creating aligned text tables.
///
/// - Columns are left-aligned by default, separated by [`COLUMN_GAP`] spaces
/// - Use [`right_align`](Self::right_align) to right-align specific columns
/// - Headers remain left-aligned even when data columns are right-aligned
pub(crate) struct TableBuilder<'a> {
    headers: Option<Vec<Cow<'a, str>>>,
    rows: Vec<Vec<Cow<'a, str>>>,
    right_aligned: Vec<bool>,
}

impl<'a> TableBuilder<'a> {
    /// Create a new table builder.
    pub(crate) fn new() -> Self {
        Self {
            headers: None,
            rows: Vec::new(),
            right_aligned: Vec::new(),
        }
    }

    /// Set the header row.
    ///
    /// Uses associated type bound instead of nested `impl Trait` for IDE compatibility.
    ///
    /// - [intellij-rust#8414](https://github.com/intellij-rust/intellij-rust/issues/8414)
    pub(crate) fn headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.headers = Some(headers.into_iter().map(Into::into).collect());
        self
    }

    /// Set column alignments (true = right-aligned, false = left-aligned).
    ///
    /// Headers remain left-aligned; only data rows are affected.
    pub(crate) fn right_align(mut self, columns: Vec<bool>) -> Self {
        self.right_aligned = columns;
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
            format_row(&mut output, headers, &widths, &no_right_align);
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
            for (width, cell) in widths.iter_mut().zip(headers) {
                *width = (*width).max(cell.len());
            }
        }
        for row in &self.rows {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(cell.len());
            }
        }
        widths
    }
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
        if right_aligned.get(i).copied().unwrap_or(false) {
            let _ = write!(output, "{cell:>width$}");
        } else {
            let _ = write!(output, "{cell:<width$}");
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
}
