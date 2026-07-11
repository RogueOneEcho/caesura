use crate::testing_prelude::*;

/// A valid UTF-8 string renders verbatim.
#[test]
fn raw_string_to_display_valid() {
    let raw = RawString::String("song.flac".to_owned());
    assert_eq!(raw.to_string_with_hex(), "song.flac");
}

/// A single invalid byte renders as its uppercase hex value.
#[test]
fn raw_string_to_display_invalid_byte() {
    let raw = RawString::Bytes(splice(b"song", E_ACUTE, b""));
    assert_eq!(raw.to_string_with_hex(), "song<E9>");
}

/// Consecutive invalid bytes each render as their own hex value.
#[test]
fn raw_string_to_display_consecutive_invalid_bytes() {
    let raw = RawString::Bytes(vec![0xE9, 0xBF]);
    assert_eq!(raw.to_string_with_hex(), "<E9><BF>");
}
