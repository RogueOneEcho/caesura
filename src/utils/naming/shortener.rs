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
        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/d"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );
        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );

        let paths = vec![PathBuf::from("a/b/c"), PathBuf::from("a/b/c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b/c")
        );

        let paths = vec![PathBuf::from("a/b/c"), PathBuf::from("x/y")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        let paths = vec![];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        let paths = vec![PathBuf::from("")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from(""),
            PathBuf::from(""),
            PathBuf::from(""),
        ];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        // Absolute paths share root
        let paths = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("/"));

        // Relative paths have no common prefix
        let paths = vec![PathBuf::from("a"), PathBuf::from("b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        // Current directory variants
        let paths = vec![PathBuf::from(".")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("."));

        let paths = vec![PathBuf::from("./")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("./")
        );

        let paths = vec![PathBuf::from("."), PathBuf::from(".")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("."));

        let paths = vec![PathBuf::from("./a"), PathBuf::from("./b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("."));

        let paths = vec![PathBuf::from("./a/b"), PathBuf::from("./a/c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("./a")
        );

        // Mixed current directory and relative
        let paths = vec![PathBuf::from("./a"), PathBuf::from("a")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        // Parent directory
        let paths = vec![PathBuf::from("../a"), PathBuf::from("../b")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("..")
        );

        let paths = vec![PathBuf::from(".."), PathBuf::from("..")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("..")
        );

        // Home directory (tilde is not expanded by PathBuf)
        let paths = vec![PathBuf::from("~/a"), PathBuf::from("~/b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("~"));

        let paths = vec![PathBuf::from("~"), PathBuf::from("~")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("~"));

        // Paths with embedded parent references (not canonicalized)
        let paths = vec![PathBuf::from("a/../b"), PathBuf::from("a/../c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/..")
        );

        let paths = vec![PathBuf::from("a/b/../c"), PathBuf::from("a/b/../d")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b/..")
        );

        // Paths with embedded current directory references
        let paths = vec![PathBuf::from("a/./b"), PathBuf::from("a/./c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/.")
        );

        // Mixed weird paths
        let paths = vec![PathBuf::from("a/../b"), PathBuf::from("a/b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("a"));

        // Double dots in sequence
        let paths = vec![PathBuf::from("a/../../b"), PathBuf::from("a/../../c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/../..")
        );

        // Trailing slashes
        let paths = vec![PathBuf::from("a/b/"), PathBuf::from("a/b/")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b/")
        );

        let paths = vec![PathBuf::from("a/b/"), PathBuf::from("a/b")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );

        // Different roots with same structure
        let paths = vec![PathBuf::from("a/../b"), PathBuf::from("c/../b")];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from(""));

        // One path is prefix of another
        let paths = vec![PathBuf::from("a/b"), PathBuf::from("a/b/c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );

        let paths = vec![PathBuf::from("a/b/c"), PathBuf::from("a/b")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );

        // More than two paths
        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/d"),
            PathBuf::from("a/b/e"),
            PathBuf::from("a/b/f"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b")
        );

        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/d"),
            PathBuf::from("a/x/e"),
        ];
        assert_eq!(Shortener::longest_common_prefix(&paths), PathBuf::from("a"));

        // Paths with spaces
        let paths = vec![PathBuf::from("a b/c d"), PathBuf::from("a b/e f")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a b")
        );

        // Unicode paths
        let paths = vec![
            PathBuf::from("音楽/アルバム/曲.flac"),
            PathBuf::from("音楽/アルバム/別曲.flac"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("音楽/アルバム")
        );

        let paths = vec![
            PathBuf::from("музика/альбом"),
            PathBuf::from("музика/інший"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("музика")
        );

        // All identical paths
        let paths = vec![
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/c"),
            PathBuf::from("a/b/c"),
        ];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b/c")
        );

        // Single path returns itself
        // TODO While this is understandable I think it may cause issues.
        let paths = vec![PathBuf::from("a/b/c")];
        assert_eq!(
            Shortener::longest_common_prefix(&paths),
            PathBuf::from("a/b/c")
        );
    }
}
