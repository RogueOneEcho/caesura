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
        let docs = OptionsRegistration::get_all();

        let mut out = String::from(concat!(
            "# Options Reference\n\n",
            "This document describes all configuration options available in caesura.\n",
            "Options can be set via CLI flags or in `config.yml`.\n\n",
        ));

        for doc in docs {
            out.push_str(&render_options_table(doc));
        }
        out.truncate(out.trim_end().len());
        out.push('\n');
        out
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

/// Format a list of commands as a bulleted markdown list.
fn format_commands(commands: &[Command]) -> String {
    commands
        .iter()
        .map(|cmd| format!("- `{}`", cmd.doc_name()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_options_table(doc: &OptionsDoc) -> String {
    let header = if doc.description.is_empty() {
        doc.name
    } else {
        doc.description
    };
    let commands = commands_for_options(doc.name);
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
    let mut output = format!("## {header}\n\n");
    if !commands.is_empty() {
        let _ = writeln!(output, "Commands:\n{}\n", format_commands(&commands));
    }
    let _ = writeln!(output, "{}", table.build());
    output
}
