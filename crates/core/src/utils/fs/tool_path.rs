//! Convert paths into arguments for external tools.

use crate::prelude::*;

/// Convert a path into arguments that `sox_ng`, `flac` and `lame` can open.
///
/// - Windows tools can't open plain paths over `MAX_PATH` unless they use the `\\?\` prefix
/// - Paths are returned unchanged on other platforms
pub(crate) trait ToolPathExt {
    /// Path to pass to an external tool.
    ///
    /// - On Windows, paths of 248 or more UTF-16 code units are made absolute and prefixed with `\\?\`
    /// - Network share paths are prefixed with `\\?\UNC\`
    fn to_tool_path(&self) -> PathBuf;

    /// Arguments to pass a `sox_ng` input or output file.
    ///
    /// - On Windows, long paths are preceded by `--no-glob` because `sox_ng` fails to glob them
    /// - `sox_ng` globs output files too, NOT only inputs
    fn to_sox_file_args(&self) -> Vec<String>;
}

impl ToolPathExt for Path {
    fn to_tool_path(&self) -> PathBuf {
        #[cfg(windows)]
        if self.exceeds_max_path() {
            return self.to_verbatim_path();
        }
        self.to_path_buf()
    }

    fn to_sox_file_args(&self) -> Vec<String> {
        let path = self.to_tool_path().to_string_lossy().to_string();
        #[cfg(windows)]
        if self.exceeds_max_path() {
            return vec!["--no-glob".to_owned(), path];
        }
        vec![path]
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use crate::testing_prelude::*;

    /// Long paths are passed through unchanged off Windows.
    #[test]
    fn path_to_tool_path_long() {
        // Arrange
        let path = PathBuf::from(format!("/{}", long_path()));
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(output, path);
    }

    /// Long paths are passed to `sox_ng` without `--no-glob` off Windows.
    #[test]
    fn path_to_sox_file_args_long() {
        // Arrange
        let path = PathBuf::from(format!("/{}", long_path()));
        // Act
        let output = path.to_sox_file_args();
        // Assert
        assert_eq!(output, vec![path.to_string_lossy().to_string()]);
    }

    /// Relative path of 49-character folder names over 300 characters long.
    fn long_path() -> String {
        vec!["a".repeat(49); 7].join("/")
    }
}
