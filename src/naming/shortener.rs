use crate::fs::FlacFile;
use crate::naming::{SourceName, TrackName};
use crate::source::{Metadata, Source};
use colored::Colorize;
use log::info;
use regex::Regex;
use rogue_logging::Colors;

pub struct Shortener;

impl Shortener {
    #[must_use]
    pub fn shorten_album(metadata: &Metadata) -> Option<Metadata> {
        let result = remove_parenthetical_suffix(&metadata.album);
        match result {
            None => None,
            Some(album) => {
                let mut metadata = metadata.clone();
                metadata.album = album;
                Some(metadata)
            }
        }
    }

    pub fn suggest_track_name(flac: &FlacFile) {
        if let Some(file_name) = TrackName::get(flac) {
            let difference = compare_char_count(&flac.file_name, &file_name);
            if difference < 0 {
                info!(
                    "{} track could save {} characters: {}",
                    "Renaming".bold(),
                    difference * -1,
                    file_name.gray()
                );
            }
        }
    }

    pub fn suggest_album_name(source: &Source) {
        if let Some(shortened) = Shortener::shorten_album(&source.metadata) {
            let before = SourceName::get(&source.metadata);
            let after = SourceName::get(&shortened);
            let difference = before.len() - after.len();
            if difference > 0 {
                info!(
                    "{} directory could save {difference} characters: {}",
                    "Renaming".bold(),
                    after.gray()
                );
            }
        }
    }
}

#[allow(clippy::if_then_some_else_none)]
fn remove_parenthetical_suffix(input: &str) -> Option<String> {
    let captures = Regex::new(r"^(.*)(\(.*\))$")
        .expect("Regex should compile")
        .captures(input)?;
    let shortened = captures.get(1).expect("Should have captures").as_str();
    let shortened = shortened.trim();
    if shortened.len() > 4 {
        Some(shortened.to_owned())
    } else {
        None
    }
}

#[allow(clippy::as_conversions, clippy::cast_possible_wrap)]
fn compare_char_count(before: &str, after: &str) -> isize {
    after.chars().count() as isize - before.chars().count() as isize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_char_count_tests() {
        assert_eq!(compare_char_count("123", "123"), 0);
        assert_eq!(compare_char_count("123", "1234"), 1);
        assert_eq!(compare_char_count("1234", "123"), -1);
        assert_eq!(
            compare_char_count("こんにちは世界！", "こんにちは世界！"),
            0
        );
        assert_eq!(compare_char_count("こんにちは世界！", "こんにちは世！"), -1);
        assert_eq!(compare_char_count("😀🙃", "😀"), -1);
        assert_eq!(compare_char_count("a\u{300}", ""), -2);
        assert_eq!(compare_char_count("\u{e0}", ""), -1);
    }
}
