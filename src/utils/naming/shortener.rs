use std::path::PathBuf;

use colored::Colorize;
use log::info;
use regex::Regex;
use rogue_logging::Colors;

use crate::utils::*;
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
                    "{} track could save {} characters:\n{}",
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
            let difference = compare_char_count(&before, &after);
            if difference < 0 {
                info!(
                    "{} directory could save {} characters: {}",
                    "Renaming".bold(),
                    difference * -1,
                    after.gray()
                );
            }
        }
    }

    #[must_use]
    pub fn longest_common_prefix(paths: &[PathBuf]) -> PathBuf {
        if let Some(first) = paths.first() {
            let mut prefix = first.clone();
            for path in paths.iter().skip(1) {
                while !path.starts_with(&prefix) {
                    if !prefix.pop() {
                        return PathBuf::new();
                    }
                }
            }
            prefix
        } else {
            PathBuf::new()
        }
    }
}

fn remove_parenthetical_suffix(input: &str) -> Option<String> {
    let captures = Regex::new(r"^(.+)\(.+\)$")
        .expect("regex should compile")
        .captures(input.trim())?;
    let shortened = captures.get(1).expect("should have captures").as_str();
    Some(shortened.trim().to_owned())
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

    #[test]
    fn remove_parenthetical_suffix_tests() {
        assert_eq!(
            remove_parenthetical_suffix("abc (123)"),
            Some("abc".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("abc (xyz)"),
            Some("abc".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("hello world (2023)"),
            Some("hello world".to_owned())
        );
        assert_eq!(remove_parenthetical_suffix("abc()"), None);
        assert_eq!(remove_parenthetical_suffix("(123)"), None);
        assert_eq!(remove_parenthetical_suffix("()"), None);
        assert_eq!(remove_parenthetical_suffix("abc"), None);
        assert_eq!(remove_parenthetical_suffix(""), None);
        assert_eq!(
            remove_parenthetical_suffix("abc  (123)"),
            Some("abc".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("   abc (123)   "),
            Some("abc".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("こんにちは (世界)"),
            Some("こんにちは".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("😀🙃 (emoji)"),
            Some("😀🙃".to_owned())
        );
        assert_eq!(
            remove_parenthetical_suffix("a!@#$%^&*() (123)"),
            Some("a!@#$%^&*()".to_owned())
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn shorten_album() {
        // Arrange
        let metadata = Metadata {
            artist: "Artist Name".to_owned(),
            album: "This is a Long Title (With an Even Longer Paranthetical Statement)".to_owned(),
            remaster_title: "Remaster Title".to_owned(),
            year: 1234,
            media: "Vinyl".to_owned(),
        };

        // Act
        let result = Shortener::shorten_album(&metadata);

        // Assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().album, "This is a Long Title");
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn longest_common_prefix_tests() {
        // Two paths with shared prefix
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d")]),
            p("a/b")
        );

        // Three paths with shared prefix (one with trailing slash)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c"), p("a/b/")]),
            p("a/b")
        );

        // Identical paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c")]),
            p("a/b/c")
        );

        // No shared prefix
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("x/y")]),
            empty()
        );

        // Empty input
        assert_eq!(Shortener::longest_common_prefix(&[]), empty());

        // Single empty path
        assert_eq!(Shortener::longest_common_prefix(&[empty()]), empty());

        // First path valid, rest empty
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), empty(), empty(), empty()]),
            empty()
        );

        // Absolute paths share root
        assert_eq!(
            Shortener::longest_common_prefix(&[p("/a"), p("/b")]),
            p("/")
        );

        // Relative paths have no common prefix
        assert_eq!(Shortener::longest_common_prefix(&[p("a"), p("b")]), empty());

        // Current directory variants
        assert_eq!(Shortener::longest_common_prefix(&[p(".")]), p("."));
        assert_eq!(Shortener::longest_common_prefix(&[p("./")]), p("./"));
        assert_eq!(Shortener::longest_common_prefix(&[p("."), p(".")]), p("."));
        assert_eq!(
            Shortener::longest_common_prefix(&[p("./a"), p("./b")]),
            p(".")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("./a/b"), p("./a/c")]),
            p("./a")
        );

        // Mixed current directory and relative
        assert_eq!(
            Shortener::longest_common_prefix(&[p("./a"), p("a")]),
            empty()
        );

        // Parent directory
        assert_eq!(
            Shortener::longest_common_prefix(&[p("../a"), p("../b")]),
            p("..")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p(".."), p("..")]),
            p("..")
        );

        // Home directory (tilde is not expanded by PathBuf)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("~/a"), p("~/b")]),
            p("~")
        );
        assert_eq!(Shortener::longest_common_prefix(&[p("~"), p("~")]), p("~"));

        // Paths with embedded parent references (not canonicalized)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("a/../c")]),
            p("a/..")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/../c"), p("a/b/../d")]),
            p("a/b/..")
        );

        // Paths with embedded current directory references
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/./b"), p("a/./c")]),
            p("a/.")
        );

        // Mixed weird paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("a/b")]),
            p("a")
        );

        // Double dots in sequence
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../../b"), p("a/../../c")]),
            p("a/../..")
        );

        // Trailing slashes
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/"), p("a/b/")]),
            p("a/b/")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/"), p("a/b")]),
            p("a/b")
        );

        // Different roots with same structure
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("c/../b")]),
            empty()
        );

        // One path is prefix of another
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b"), p("a/b/c")]),
            p("a/b")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b")]),
            p("a/b")
        );

        // More than two paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d"), p("a/b/e"), p("a/b/f")]),
            p("a/b")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d"), p("a/x/e")]),
            p("a")
        );

        // Paths with spaces
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a b/c d"), p("a b/e f")]),
            p("a b")
        );

        // Unicode paths
        assert_eq!(
            Shortener::longest_common_prefix(&[
                p("音楽/アルバム/曲.flac"),
                p("音楽/アルバム/別曲.flac")
            ]),
            p("音楽/アルバム")
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("музика/альбом"), p("музика/інший")]),
            p("музика")
        );

        // All identical paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c"), p("a/b/c")]),
            p("a/b/c")
        );

        // Single path returns itself
        assert_eq!(Shortener::longest_common_prefix(&[p("a/b/c")]), p("a/b/c"));
    }

    fn p(path: &str) -> PathBuf {
        PathBuf::from(path)
    }

    fn empty() -> PathBuf {
        PathBuf::new()
    }
}
