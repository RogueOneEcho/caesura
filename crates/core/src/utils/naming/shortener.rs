use std::path::Path;

use crate::prelude::*;

/// Suggest shorter names for albums and tracks.
pub struct Shortener;

impl Shortener {
    /// Find the longest path prefix shared by all paths.
    ///
    /// Starts with the first path and progressively shortens it until all paths match.
    ///
    /// Returns `None` if paths is empty or no common prefix exists.
    ///
    /// Note: no canonicalization is applied.
    #[must_use]
    pub fn longest_common_prefix(paths: &[impl AsRef<Path>]) -> Option<PathBuf> {
        let first = paths.first()?;
        let mut prefix = first.as_ref().to_path_buf();
        for path in paths.iter().skip(1) {
            while !path.as_ref().starts_with(&prefix) {
                if !prefix.pop() {
                    return None;
                }
            }
        }
        if prefix.as_os_str().is_empty() {
            None
        } else {
            Some(prefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn longest_common_prefix_tests() {
        // Two paths with shared prefix
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d")]),
            Some(p("a/b"))
        );

        // Three paths with shared prefix (one with trailing slash)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c"), p("a/b/")]),
            Some(p("a/b"))
        );

        // Identical paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c")]),
            Some(p("a/b/c"))
        );

        // No shared prefix
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("x/y")]),
            None
        );

        // Empty input
        assert_eq!(
            Shortener::longest_common_prefix(&Vec::<PathBuf>::new()),
            None
        );

        // Single empty path
        assert_eq!(Shortener::longest_common_prefix(&[p("")]), None);

        // First path valid, rest empty
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p(""), p(""), p("")]),
            None
        );

        // Absolute paths share root
        assert_eq!(
            Shortener::longest_common_prefix(&[p("/a"), p("/b")]),
            Some(p("/"))
        );

        // Relative paths have no common prefix
        assert_eq!(Shortener::longest_common_prefix(&[p("a"), p("b")]), None);

        // Current directory variants
        assert_eq!(Shortener::longest_common_prefix(&[p(".")]), Some(p(".")));
        assert_eq!(Shortener::longest_common_prefix(&[p("./")]), Some(p("./")));
        assert_eq!(
            Shortener::longest_common_prefix(&[p("."), p(".")]),
            Some(p("."))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("./a"), p("./b")]),
            Some(p("."))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("./a/b"), p("./a/c")]),
            Some(p("./a"))
        );

        // Mixed current directory and relative
        assert_eq!(Shortener::longest_common_prefix(&[p("./a"), p("a")]), None);

        // Parent directory
        assert_eq!(
            Shortener::longest_common_prefix(&[p("../a"), p("../b")]),
            Some(p(".."))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p(".."), p("..")]),
            Some(p(".."))
        );

        // Home directory (tilde is not expanded by PathBuf)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("~/a"), p("~/b")]),
            Some(p("~"))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("~"), p("~")]),
            Some(p("~"))
        );

        // Paths with embedded parent references (not canonicalized)
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("a/../c")]),
            Some(p("a/.."))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/../c"), p("a/b/../d")]),
            Some(p("a/b/.."))
        );

        // Paths with embedded current directory references
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/./b"), p("a/./c")]),
            Some(p("a/."))
        );

        // Mixed weird paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("a/b")]),
            Some(p("a"))
        );

        // Double dots in sequence
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../../b"), p("a/../../c")]),
            Some(p("a/../.."))
        );

        // Trailing slashes
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/"), p("a/b/")]),
            Some(p("a/b/"))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/"), p("a/b")]),
            Some(p("a/b"))
        );

        // Different roots with same structure
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/../b"), p("c/../b")]),
            None
        );

        // One path is prefix of another
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b"), p("a/b/c")]),
            Some(p("a/b"))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b")]),
            Some(p("a/b"))
        );

        // More than two paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d"), p("a/b/e"), p("a/b/f")]),
            Some(p("a/b"))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/d"), p("a/x/e")]),
            Some(p("a"))
        );

        // Paths with spaces
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a b/c d"), p("a b/e f")]),
            Some(p("a b"))
        );

        // Unicode paths
        assert_eq!(
            Shortener::longest_common_prefix(&[
                p("音楽/アルバム/曲.flac"),
                p("音楽/アルバム/別曲.flac")
            ]),
            Some(p("音楽/アルバム"))
        );
        assert_eq!(
            Shortener::longest_common_prefix(&[p("музика/альбом"), p("музика/інший")]),
            Some(p("музика"))
        );

        // All identical paths
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c"), p("a/b/c"), p("a/b/c")]),
            Some(p("a/b/c"))
        );

        // Single path returns itself
        assert_eq!(
            Shortener::longest_common_prefix(&[p("a/b/c")]),
            Some(p("a/b/c"))
        );
    }

    fn p(path: &str) -> PathBuf {
        PathBuf::from(path)
    }
}
