//! Implementation of the `test-name` command.

use std::fmt::Write;

use crate::prelude::*;
use regex::Regex;

/// Display resolved output names using mock metadata.
#[injectable]
pub(crate) struct TestNameCommand {
    /// Naming options from CLI and config file.
    options: Ref<NameOptions>,
    /// Resolver that renders templates into folder names.
    name_resolver: Ref<NameResolver>,
}

impl TestNameCommand {
    /// Execute the command, printing resolved names to stdout.
    pub fn execute(&self) -> bool {
        let mut output = String::new();
        let template = self
            .options
            .name_template
            .as_deref()
            .unwrap_or(DEFAULT_TEMPLATE);
        writeln!(output, "{}", "Template".green().dimmed())
            .expect("write to string should succeed");
        writeln!(output, "{}", colorize_template(template))
            .expect("write to string should succeed");
        for format in [TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0] {
            let value = self.name_resolver.transcode(&Metadata::mock(), format);
            write(
                &mut output,
                format!("{} transcode", format.get_name()),
                &value,
            );
        }
        let value = self.name_resolver.spectrogram(&Metadata::mock());
        write(&mut output, "Spectrogram", &value);
        let value = self.name_resolver.spectrogram(&Metadata {
            edition_title: None,
            ..Metadata::mock()
        });
        write(&mut output, "Without edition_title", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_2_artists(), TargetFormat::Flac);
        write(&mut output, "2 artists", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_3_artists(), TargetFormat::Flac);
        write(&mut output, "3 artists, no fallback", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_3_artists_1_dj(), TargetFormat::Flac);
        write(&mut output, "3 artists, 1 DJ", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_3_artists_1_composer(), TargetFormat::Flac);
        write(&mut output, "3 artists, 1 composer", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_1_artist_1_composer(), TargetFormat::Flac);
        write(&mut output, "1 artist, 1 composer", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_0_artists_2_guests(), TargetFormat::Flac);
        write(&mut output, "0 artists, 2 guests", &value);
        let value = self
            .name_resolver
            .transcode(&Metadata::mock_0_artists(), TargetFormat::Flac);
        write(&mut output, "0 artists, 0 guests", &value);
        eprintln!("{output}");
        true
    }
}

/// Colorize a minijinja template for terminal display.
fn colorize_template(template: &str) -> String {
    let value_re = Regex::new(r"\{\{.*?\}\}").expect("value regex should compile");
    let tag_re = Regex::new(r"\{%.*?%\}").expect("tag regex should compile");
    let result = value_re.replace_all(template, |caps: &regex::Captures| {
        caps[0].yellow().to_string()
    });
    tag_re
        .replace_all(&result, |caps: &regex::Captures| caps[0].cyan().to_string())
        .into_owned()
}

/// Append a labeled entry with a green dimmed label and plain value.
fn write(output: &mut String, label: impl Into<String>, value: impl Into<String>) {
    writeln!(output, "\n{}", label.into().green().dimmed())
        .expect("write to string should succeed");
    writeln!(output, "{}", value.into().yellow()).expect("write to string should succeed");
}
