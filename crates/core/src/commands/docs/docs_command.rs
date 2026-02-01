use crate::prelude::*;

/// Generate documentation for configuration options.
#[injectable]
pub struct DocsCommand;

impl DocsCommand {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn execute(&self) -> Result<bool, Error> {
        print!("{}", Self::render());
        Ok(true)
    }

    /// Render all options documentation as markdown.
    #[must_use]
    pub fn render() -> String {
        let docs: Vec<&OptionsDoc> = vec![
            SharedOptions::doc_metadata(),
            VerifyOptions::doc_metadata(),
            TargetOptions::doc_metadata(),
            SpectrogramOptions::doc_metadata(),
            CopyOptions::doc_metadata(),
            FileOptions::doc_metadata(),
            RunnerOptions::doc_metadata(),
            UploadOptions::doc_metadata(),
            CacheOptions::doc_metadata(),
            BatchOptions::doc_metadata(),
        ];

        let mut out = String::new();
        out.push_str("# Configuration Reference\n\n");
        out.push_str("This document describes all configuration options available in caesura.\n");
        out.push_str("Options can be set via CLI flags or in `config.yml`.\n\n");

        for doc in docs {
            out.push_str(&render_options_table(doc));
        }

        out
    }
}

fn render_options_table(doc: &OptionsDoc) -> String {
    let mut out = String::new();
    let header = if doc.description.is_empty() {
        doc.name
    } else {
        doc.description
    };
    let _ = writeln!(out, "## {header}\n");

    // Prepare rows data
    let headers = ["YAML Key", "CLI Flag", "Type", "Default", "Description"];
    let rows: Vec<[String; 5]> = doc
        .fields
        .iter()
        .map(|field| {
            let default = if let Some(doc) = field.default_doc {
                doc.to_owned()
            } else if let Some(value) = &field.default_value {
                format!("`{value}`")
            } else {
                "~".to_owned()
            };
            let description = escape_markdown_table(field.description);
            [
                format!("`{}`", field.config_key),
                format!("`{}`", field.cli_flag),
                format!("`{}`", field.field_type),
                default,
                description,
            ]
        })
        .collect();

    // Calculate column widths
    let mut widths = headers.map(str::len);
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row.iter()) {
            *width = (*width).max(cell.len());
        }
    }

    // Render header row
    out.push('|');
    for (h, width) in headers.iter().zip(widths.iter()) {
        let _ = write!(out, " {h:<width$} |");
    }
    out.push('\n');

    // Render separator row
    out.push('|');
    for width in widths {
        let _ = write!(out, "-{:-<width$}-|", "");
    }
    out.push('\n');

    // Render data rows
    for row in &rows {
        out.push('|');
        for (cell, width) in row.iter().zip(widths.iter()) {
            let _ = write!(out, " {cell:<width$} |");
        }
        out.push('\n');
    }

    out.push('\n');
    out
}

/// Escapes pipe characters and newlines for markdown table cells.
fn escape_markdown_table(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}
