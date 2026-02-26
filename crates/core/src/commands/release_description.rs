use crate::prelude::*;

/// Append inspect-derived `Details` and `Tags` sections to `BBCode` description lines.
pub(crate) fn append_inspect_sections(
    lines: &mut Vec<String>,
    source_path: &Path,
    warning_context: &str,
) {
    let factory = InspectFactory::new(false);
    match factory.create_split(source_path) {
        Ok((properties, tags)) => {
            lines.push(format!(
                "[pad=0|10|0|19]Details[/pad] [pre]{properties}[/pre]"
            ));
            lines.push(format!(
                "[pad=0|10|0|31]Tags[/pad] [hide][pre]{tags}[/pre][/hide]"
            ));
        }
        Err(e) => {
            warn!("{warning_context}\n{}", e.render());
        }
    }
}

/// Render description lines into `[quote]...[/quote]` `BBCode` blocks.
#[must_use]
pub(crate) fn to_quote_blocks(lines: Vec<String>) -> String {
    lines.into_iter().fold(String::new(), |mut output, line| {
        output.push_str("[quote]");
        output.push_str(&line);
        output.push_str("[/quote]");
        output
    })
}
