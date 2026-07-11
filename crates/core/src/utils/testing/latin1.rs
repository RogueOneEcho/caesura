//! Latin-1 bytes used to build non-UTF-8 torrent fixtures.
//!
//! Each byte is a valid Windows-1252 (Latin-1) accented letter but not valid
//! UTF-8, so it also acts as a UTF-8 lead byte when libtorrent decodes it.

/// Latin-1 `é`
///
/// - A UTF-8 three-byte lead.
pub(crate) const E_ACUTE: u8 = 0xE9;

/// Latin-1 `í`
///
/// - A UTF-8 three-byte lead that consumes the next two bytes.
pub(crate) const I_ACUTE: u8 = 0xED;

/// Latin-1 `ö`
///
/// - A UTF-8 four-byte lead that consumes the next three bytes.
pub(crate) const O_DIAERESIS: u8 = 0xF6;

/// Latin-1 `ü`
///
/// -An invalid UTF-8 lead skipped as a single byte.
pub(crate) const U_DIAERESIS: u8 = 0xFC;

/// Splice `byte` between `prefix` and `suffix` as a byte string.
pub(crate) fn splice(prefix: &[u8], byte: u8, suffix: &[u8]) -> Vec<u8> {
    [prefix, &[byte], suffix].concat()
}
