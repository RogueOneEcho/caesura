use crate::prelude::*;

/// Generate documentation for configuration options.
#[injectable]
pub struct DocsCommand;

impl DocsCommand {
    /// Print rendered options documentation to stdout.
    #[allow(clippy::unused_self)]
    pub fn execute(&self) -> bool {
        print!("{}", Self::render());
        true
    }

    /// Render all options documentation as markdown.
    #[must_use]
    pub fn render() -> String {
        let docs: Vec<&OptionsDoc> = vec![
            ConfigOptions::doc_metadata(),
            SharedOptions::doc_metadata(),
            VerifyOptions::doc_metadata(),
            TargetOptions::doc_metadata(),
            SpectrogramOptions::doc_metadata(),
            SoxOptions::doc_metadata(),
            CopyOptions::doc_metadata(),
            FileOptions::doc_metadata(),
            RunnerOptions::doc_metadata(),
            UploadOptions::doc_metadata(),
            CacheOptions::doc_metadata(),
            BatchOptions::doc_metadata(),
            QueueAddArgs::doc_metadata(),
        ];

        let mut out = String::from(concat!(
            "# Options Reference\n\n",
            "This document describes all configuration options available in caesura.\n",
            "Options can be set via CLI flags or in `config.yml`.\n\n",
        ));

        for doc in docs {
            out.push_str(&render_options_table(doc));
        }

        out
    }
}

fn render_options_table(doc: &OptionsDoc) -> String {
    let header = if doc.description.is_empty() {
        doc.name
    } else {
        doc.description
    };
    let mut table = TableBuilder::new().markdown().headers([
        "YAML Key",
        "CLI Flag",
        "Type",
        "Default",
        "Description",
    ]);
    for field in &doc.fields {
        let default = if let Some(doc) = field.default_doc {
            doc.to_owned()
        } else if let Some(value) = &field.default_value {
            format!("`{value}`")
        } else {
            "~".to_owned()
        };
        let cli_flag = if field.cli_flag.is_empty() {
            "~".to_owned()
        } else {
            format!("`{}`", field.cli_flag)
        };
        table = table.row([
            format!("`{}`", field.config_key),
            cli_flag,
            format!("`{}`", field.field_type),
            default,
            field.description.to_owned(),
        ]);
    }
    format!("## {header}\n\n{}\n", table.build())
}
