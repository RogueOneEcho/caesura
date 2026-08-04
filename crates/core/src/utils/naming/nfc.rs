//! Normalize paths and strings to composed UTF-8 (NFC).

use crate::prelude::*;
use unicode_normalization::{UnicodeNormalization, is_nfc};

/// Compose a string into Unicode NFC form.
pub(crate) trait StrNfcExt {
    /// Replace decomposed (NFD) sequences with composed (NFC)
    ///
    /// - Composes decomposed sequences, so `e` followed by `◌́` becomes `é`
    /// - Returns a copy unchanged when the string is already composed
    fn to_nfc(&self) -> String;
}

impl StrNfcExt for str {
    fn to_nfc(&self) -> String {
        if is_nfc(self) {
            return self.to_owned();
        }
        self.nfc().collect()
    }
}

/// Compose a path into Unicode NFC form.
pub(crate) trait PathNfcExt {
    /// Replace decomposed (NFD) sequences with composed (NFC)
    ///
    /// - Composes decomposed sequences, so `e` followed by `◌́` becomes `é`
    /// - Invalid UTF-8 is replaced with `�`
    fn to_nfc(&self) -> PathBuf;
}

impl PathNfcExt for Path {
    fn to_nfc(&self) -> PathBuf {
        PathBuf::from(self.to_string_lossy().to_nfc())
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    #[test]
    fn str_to_nfc_ascii() {
        assert_eq!("Hello, world!".to_nfc(), "Hello, world!");
    }

    #[test]
    fn str_to_nfc_composed() {
        assert_eq!("Caf\u{e9}".to_nfc(), "Caf\u{e9}");
    }

    #[test]
    fn str_to_nfc_decomposed() {
        assert_eq!("Cafe\u{301}".to_nfc(), "Caf\u{e9}");
    }

    #[test]
    fn path_to_nfc_decomposed() {
        // Arrange
        let path = PathBuf::from("No\u{308}rd/Sce\u{301}ne.flac");
        // Act
        let output = path.to_nfc();
        // Assert
        assert_eq!(output, PathBuf::from("N\u{f6}rd/Sc\u{e9}ne.flac"));
    }

    /// A lone continuation byte is invalid UTF-8.
    #[cfg(unix)]
    #[test]
    fn path_to_nfc_non_utf8_unix() {
        // Arrange
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let path = PathBuf::from(OsStr::from_bytes(&[0x43, 0x61, 0x66, 0x80]));
        // Act
        let output = path.to_nfc();
        // Assert
        assert_eq!(output, PathBuf::from("Caf\u{fffd}"));
    }

    /// A lone surrogate half is invalid UTF-16.
    #[cfg(windows)]
    #[test]
    fn path_to_nfc_non_utf8_windows() {
        // Arrange
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let path = PathBuf::from(OsString::from_wide(&[0x0043, 0x0061, 0x0066, 0xD800]));
        // Act
        let output = path.to_nfc();
        // Assert
        assert_eq!(output, PathBuf::from("Caf\u{fffd}"));
    }
}
