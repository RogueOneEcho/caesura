//! Measure the visible width of a string in terminal columns.

use crate::prelude::*;
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// Visible width of a string in terminal columns.
///
/// - Strips ANSI SGR escape sequences before measuring
/// - Uses [`UnicodeWidthStr`] for accurate column widths (e.g., CJK characters
///   occupy 2 columns, emoji occupy 2 columns, zero-width characters occupy 0)
pub(crate) fn visible_width(s: &str) -> usize {
    strip_ansi(s).width()
}

/// Remove ANSI SGR escape sequences from a string.
pub(crate) fn strip_ansi(s: &str) -> Cow<'_, str> {
    static ANSI_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").expect("ANSI regex should be valid"));
    ANSI_RE.replace_all(s, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_width_plain_text() {
        assert_eq!(visible_width("hello"), 5);
    }

    #[test]
    fn visible_width_empty_string() {
        assert_eq!(visible_width(""), 0);
    }

    #[test]
    fn visible_width_single_ansi_code() {
        assert_eq!(visible_width("\x1b[31mhello\x1b[0m"), 5);
    }

    #[test]
    fn visible_width_multiple_ansi_codes() {
        assert_eq!(visible_width("\x1b[1m\x1b[31merror\x1b[0m"), 5);
    }

    #[test]
    fn visible_width_ansi_with_semicolons() {
        assert_eq!(visible_width("\x1b[1;31mwarning\x1b[0m"), 7);
    }

    #[test]
    fn visible_width_mixed_plain_and_ansi() {
        assert_eq!(visible_width("plain \x1b[2mdimmed\x1b[0m end"), 16);
    }

    #[test]
    fn visible_width_multibyte_utf8() {
        assert_eq!(visible_width("⚠"), 1);
    }

    #[test]
    fn visible_width_multibyte_utf8_with_ansi() {
        assert_eq!(visible_width("\x1b[31m⚠\x1b[0m"), 1);
    }

    #[test]
    fn visible_width_emoji() {
        assert_eq!(visible_width("✅"), 2);
        assert_eq!(visible_width("🎵"), 2);
    }

    #[test]
    fn visible_width_emoji_with_ansi() {
        assert_eq!(visible_width("\x1b[33m🎵\x1b[0m"), 2);
    }
}
