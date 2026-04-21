//! OSC 8 terminal hyperlink support.

use crate::prelude::*;
use colored::control::SHOULD_COLORIZE;

/// Extension trait for wrapping display text in an OSC 8 terminal hyperlink.
///
/// Respects the same `NO_COLOR`, `CLICOLOR`, and tty checks that
/// the `colored` crate uses. When colorization is disabled the
/// text is returned unchanged.
pub(crate) trait Hyperlink: Display {
    /// Wrap in an OSC 8 hyperlink to `url`.
    fn hyperlink(&self, url: &str) -> String {
        if SHOULD_COLORIZE.should_colorize() {
            format!("\x1b]8;;{url}\x1b\\{self}\x1b]8;;\x1b\\")
        } else {
            self.to_string()
        }
    }
}

impl<T: Display> Hyperlink for T {}
