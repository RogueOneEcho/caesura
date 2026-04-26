//! Word-wrap a single line to a maximum visible width.

use crate::prelude::*;
use std::mem::take;
use unicode_width::UnicodeWidthChar;

/// Wrap `line` so each output line's visible width is at most `max_width`.
///
/// - Splits on ASCII whitespace and rejoins words with single spaces
/// - Hard-breaks any word wider than `max_width` at the column limit
/// - Returns `line` unchanged when `max_width` is `0` or `line` already fits
/// - Measures width via [`visible_width`], ignoring ANSI escape sequences
pub(crate) fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 || visible_width(line) <= max_width {
        return vec![line.to_owned()];
    }
    let mut output = Vec::new();
    let mut current = String::new();
    for word in split_to_width(line, max_width) {
        let width = visible_width(&current) + separator_width(&current) + visible_width(&word);
        if width > max_width && !current.is_empty() {
            output.push(take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(&word);
    }
    if !current.is_empty() {
        output.push(current);
    }
    output
}

/// Width of the space inserted between `current` and a following word.
fn separator_width(current: &str) -> usize {
    if current.is_empty() { 0 } else { 1 }
}

/// Split `line` into pieces each no wider than `max_width` visible columns.
///
/// - Splits on ASCII whitespace
/// - Hard-breaks any word wider than `max_width` via [`hard_break`]
fn split_to_width(line: &str, max_width: usize) -> Vec<String> {
    let mut output = Vec::new();
    for word in line.split_whitespace() {
        if visible_width(word) <= max_width {
            output.push(word.to_owned());
        } else {
            output.extend(hard_break(word, max_width));
        }
    }
    output
}

/// Break `word` into chunks each no wider than `max_width` visible columns.
///
/// - Accumulates characters until adding the next would exceed `max_width`
/// - Uses [`UnicodeWidthChar`] for per-character width; zero-width characters
///   attach to the current chunk without advancing the running width
fn hard_break(word: &str, max_width: usize) -> Vec<String> {
    let mut output = Vec::new();
    let mut buf = String::new();
    let mut buf_width = 0;
    for ch in word.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if !buf.is_empty() && buf_width + ch_width > max_width {
            output.push(take(&mut buf));
            buf_width = 0;
        }
        buf.push(ch);
        buf_width += ch_width;
    }
    if !buf.is_empty() {
        output.push(buf);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_line_within_max_width() {
        assert_eq!(wrap_line("hello world", 20), vec!["hello world"]);
    }

    #[test]
    fn wrap_line_multiple_words_exceeding_max_width() {
        assert_eq!(
            wrap_line("the quick brown fox jumps", 10),
            vec!["the quick", "brown fox", "jumps"],
        );
    }

    #[test]
    fn wrap_line_word_longer_than_max_width() {
        assert_eq!(
            wrap_line("https://example.com/very/long/path", 10),
            vec!["https://ex", "ample.com/", "very/long/", "path"],
        );
    }

    #[test]
    fn wrap_line_oversize_word_then_short_word() {
        assert_eq!(wrap_line("abcdefgh ij", 5), vec!["abcde", "fgh", "ij"]);
    }

    #[test]
    fn wrap_line_with_ansi() {
        let red_hello = "\x1b[31mhello\x1b[0m";
        let input = format!("{red_hello} world {red_hello}");
        assert_eq!(
            wrap_line(&input, 11),
            vec![format!("{red_hello} world"), red_hello.to_owned()],
        );
    }

    #[test]
    fn wrap_line_zero_max_width() {
        assert_eq!(wrap_line("anything goes", 0), vec!["anything goes"]);
    }

    #[test]
    fn wrap_line_extra_whitespace_between_words() {
        assert_eq!(
            wrap_line("alpha   bravo\tcharlie", 10),
            vec!["alpha", "bravo", "charlie"],
        );
    }

    #[test]
    fn wrap_line_word_equals_max_width() {
        assert_eq!(wrap_line("hello", 5), vec!["hello"]);
    }

    #[test]
    fn wrap_line_wide_chars_no_whitespace() {
        assert_eq!(
            wrap_line("一二三四五六七八九十", 10),
            vec!["一二三四五", "六七八九十"],
        );
    }
}
