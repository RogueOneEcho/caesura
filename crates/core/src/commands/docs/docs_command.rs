use crate::prelude::*;

const NULL_MARKER: &str = "~";

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
        let docs = OptionsRegistration::get_all();
        let mut out = String::from(concat!(
            "# Options Reference\n\n",
            "This document describes all configuration options available in caesura.\n",
            "Options can be set via CLI flags or in `config.yml`.\n\n",
        ));
        out.push_str(&render_options_table(&docs));
        out.truncate(out.trim_end().len());
        out.push('\n');
        out
    }
}

fn render_options_table(docs: &[&OptionsDoc]) -> String {
    let all = Command::all().len() - 4;
    let mut table = TableBuilder::new().markdown().headers([
        "YAML Key / CLI Flag",
        "Type",
        "Default",
        "Description",
        "Commands",
    ]);
    let mut rows = Vec::new();
    for doc in docs {
        let commands = commands_for_options(doc.name);
        for field in &doc.fields {
            rows.push([
                format_key_flag(field),
                format_value(field.field_type),
                format_default(field),
                field.description.to_owned(),
                format_commands(&commands, all),
            ]);
        }
    }
    rows.sort_by_key(|row| row[0].clone());
    for row in rows {
        table = table.row(row);
    }
    table.build()
}

fn format_key_flag(field: &FieldDoc) -> String {
    if field.cli_flag.is_empty() {
        format_value(field.config_key)
    } else {
        format!(
            "{}<br><br>{}",
            format_value(field.config_key),
            format_value(field.cli_flag)
        )
    }
}

fn format_value(value: &str) -> String {
    format!("`{value}`")
}

fn format_json(value: &str) -> String {
    let mut value = value.to_owned();
    if value.starts_with('[') {
        value = value.replace("\",\"", "\", \"");
    }
    format_value(&value)
}

fn format_default(field: &FieldDoc) -> String {
    if let Some(doc) = field.default_doc {
        doc.to_owned()
    } else if let Some(value) = &field.default_value {
        format_json(value)
    } else {
        NULL_MARKER.to_owned()
    }
}

fn format_commands(commands: &[Command], all: usize) -> String {
    if commands.is_empty() {
        NULL_MARKER.to_owned()
    } else if commands.len() >= all {
        "All".to_owned()
    } else {
        commands
            .iter()
            .map(|c| format_value(c.doc_name()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Find which commands use a given options type name.
fn commands_for_options(name: &str) -> Vec<Command> {
    Command::all()
        .iter()
        .filter(|cmd| cmd.uses_options(name))
        .copied()
        .collect()
}
