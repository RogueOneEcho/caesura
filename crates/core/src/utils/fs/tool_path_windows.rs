//! Detect and convert Windows paths over `MAX_PATH`.

use crate::prelude::*;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Prefix, absolute};

/// Path length in UTF-16 code units from which Windows tools need a `\\?\` prefix.
///
/// - `MAX_PATH` of 260 includes the trailing null, so file paths fail from 260 code units
/// - Uses the stricter `CreateDirectoryW` limit of `MAX_PATH - 12` as a safety margin
const LEGACY_MAX_PATH: usize = 248;

/// `\\?\` prefix for drive paths.
const VERBATIM_PREFIX: &str = r"\\?\";

/// `\\?\UNC\` prefix that replaces the leading `\\` of network share paths.
const VERBATIM_UNC_PREFIX: &str = r"\\?\UNC\";

/// Detect and convert Windows paths over `MAX_PATH`.
pub(crate) trait WindowsPathExt {
    /// Is the absolute path too long for Windows tools without a `\\?\` prefix?
    ///
    /// - Counts UTF-16 code units, as Windows does, NOT bytes or characters
    fn exceeds_max_path(&self) -> bool;

    /// Absolute path with a `\\?\` or `\\?\UNC\` prefix.
    ///
    /// - Verbatim and device paths are returned unchanged
    fn to_verbatim_path(&self) -> PathBuf;
}

impl WindowsPathExt for Path {
    fn exceeds_max_path(&self) -> bool {
        let absolute = absolute(self).expect("path should not be empty");
        absolute.as_os_str().encode_wide().count() >= LEGACY_MAX_PATH
    }

    fn to_verbatim_path(&self) -> PathBuf {
        let absolute = absolute(self).expect("path should not be empty");
        let wide: Vec<u16> = match absolute.components().next() {
            Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::UNC(..)) => {
                VERBATIM_UNC_PREFIX
                    .encode_utf16()
                    .chain(absolute.as_os_str().encode_wide().skip(2))
                    .collect()
            }
            Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_)) => {
                VERBATIM_PREFIX
                    .encode_utf16()
                    .chain(absolute.as_os_str().encode_wide())
                    .collect()
            }
            _ => return absolute,
        };
        PathBuf::from(OsString::from_wide(&wide))
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;
    use std::env::current_dir;
    use std::os::windows::ffi::OsStrExt;

    #[test]
    fn path_to_tool_path_short() {
        let path = PathBuf::from(r"C:\music\01 Track.flac");
        assert_eq!(path.to_tool_path(), path);
    }

    #[test]
    fn path_to_tool_path_long_disk() {
        // Arrange
        let path = PathBuf::from(format!(r"C:\{}", ascii_segments(300)));
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(output, PathBuf::from(format!(r"\\?\{}", path.display())));
    }

    #[test]
    fn path_to_tool_path_long_unc() {
        // Arrange
        let segments = ascii_segments(300);
        let path = PathBuf::from(format!(r"\\server\share\{segments}"));
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(
            output,
            PathBuf::from(format!(r"\\?\UNC\server\share\{segments}"))
        );
    }

    #[test]
    fn path_to_tool_path_long_verbatim() {
        // Arrange
        let path = PathBuf::from(format!(r"\\?\C:\{}", ascii_segments(300)));
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(output, path);
    }

    /// Relative paths are resolved against the current directory before prefixing.
    #[test]
    fn path_to_tool_path_long_relative() {
        // Arrange
        let segments = ascii_segments(300);
        let path = PathBuf::from(&segments);
        let expected = current_dir()
            .expect("should get current directory")
            .join(&segments);
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(
            output,
            PathBuf::from(format!(r"\\?\{}", expected.display()))
        );
    }

    /// `\\?\` disables separator normalization, so `/` must become `\` before prefixing.
    #[test]
    fn path_to_tool_path_long_forward_slashes() {
        // Arrange
        let segments = ascii_segments(300);
        let path = PathBuf::from(format!("C:/{}", segments.replace('\\', "/")));
        // Act
        let output = path.to_tool_path();
        // Assert
        assert_eq!(output, PathBuf::from(format!(r"\\?\C:\{segments}")));
    }

    #[test]
    fn path_exceeds_max_path_below_threshold() {
        let path = PathBuf::from(format!(r"C:\{}", "a".repeat(244)));
        assert_eq!(utf16_len(&path), 247);
        assert!(!path.exceeds_max_path());
    }

    #[test]
    fn path_exceeds_max_path_at_threshold() {
        let path = PathBuf::from(format!(r"C:\{}", "a".repeat(245)));
        assert_eq!(utf16_len(&path), 248);
        assert!(path.exceeds_max_path());
    }

    /// CJK characters are 3 UTF-8 bytes but 1 UTF-16 code unit each.
    #[test]
    fn path_exceeds_max_path_multibyte_utf8() {
        // Arrange
        let path = PathBuf::from(format!(r"C:\{}", "日".repeat(240)));
        // Act
        let output = path.exceeds_max_path();
        // Assert
        assert!(path.to_string_lossy().len() >= 248);
        assert_eq!(utf16_len(&path), 243);
        assert!(!output);
    }

    /// Emoji are 1 character but 2 UTF-16 code units each.
    #[test]
    fn path_exceeds_max_path_surrogate_pairs() {
        // Arrange
        let path = PathBuf::from(format!(r"C:\{}", "🎵".repeat(123)));
        // Act
        let output = path.exceeds_max_path();
        // Assert
        assert_eq!(path.to_string_lossy().chars().count(), 126);
        assert_eq!(utf16_len(&path), 249);
        assert!(output);
    }

    #[test]
    fn path_to_sox_file_args_short() {
        let path = PathBuf::from(r"C:\music\01 Track.flac");
        assert_eq!(path.to_sox_file_args(), vec![r"C:\music\01 Track.flac"]);
    }

    #[test]
    fn path_to_sox_file_args_long() {
        // Arrange
        let segments = ascii_segments(300);
        let path = PathBuf::from(format!(r"C:\{segments}"));
        // Act
        let output = path.to_sox_file_args();
        // Assert
        assert_eq!(
            output,
            vec!["--no-glob".to_owned(), format!(r"\\?\C:\{segments}")]
        );
    }

    /// Join 49-character folder names with `\` to reach `length`.
    ///
    /// - Keeps each folder under the 255 character limit for a single path component
    fn ascii_segments(length: usize) -> String {
        let segment = "a".repeat(49);
        let count = length.div_ceil(segment.len() + 1);
        vec![segment; count].join(r"\")
    }

    fn utf16_len(path: &Path) -> usize {
        path.as_os_str().encode_wide().count()
    }
}
