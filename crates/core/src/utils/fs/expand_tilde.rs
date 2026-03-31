//! Tilde expansion for user-provided paths.

use std::path::{Path, PathBuf};

/// Extension trait for expanding `~` in paths.
pub(crate) trait ExpandTilde {
    /// Expand a leading `~` to the user's home directory.
    ///
    /// - `~/foo` becomes `/home/user/foo`
    /// - `~` alone becomes `/home/user`
    /// - Paths not starting with `~` are returned unchanged
    /// - `~user` syntax is not supported
    fn expand_tilde(&self) -> PathBuf;
}

impl ExpandTilde for Path {
    fn expand_tilde(&self) -> PathBuf {
        let Some(s) = self.to_str() else {
            return self.to_path_buf();
        };
        if s == "~" {
            return dirs::home_dir().expect("home directory should be determinable");
        }
        if let Some(rest) = s.strip_prefix("~/") {
            return dirs::home_dir()
                .expect("home directory should be determinable")
                .join(rest);
        }
        self.to_path_buf()
    }
}

impl ExpandTilde for PathBuf {
    fn expand_tilde(&self) -> PathBuf {
        self.as_path().expand_tilde()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_with_subpath() {
        let expanded = Path::new("~/.cache/caesura").expand_tilde();
        assert!(!expanded.starts_with("~"), "tilde should be expanded");
        assert!(
            expanded.ends_with(".cache/caesura"),
            "subpath should be preserved"
        );
    }

    #[test]
    fn expand_tilde_alone() {
        let expanded = Path::new("~").expand_tilde();
        assert!(!expanded.starts_with("~"), "tilde should be expanded");
        assert!(expanded.is_absolute(), "should be an absolute path");
    }

    #[test]
    fn expand_tilde_absolute_path_unchanged() {
        let path = Path::new("/home/user/.cache/caesura");
        assert_eq!(path.expand_tilde(), path);
    }

    #[test]
    fn expand_tilde_relative_path_unchanged() {
        let path = Path::new("relative/path");
        assert_eq!(path.expand_tilde(), path);
    }

    #[test]
    fn expand_tilde_dollar_home_unchanged() {
        let path = Path::new("$HOME/.cache/caesura");
        assert_eq!(path.expand_tilde(), path);
    }
}
