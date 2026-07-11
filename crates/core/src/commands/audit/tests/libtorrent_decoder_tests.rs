use crate::testing_prelude::*;

/// A latin-1 byte immediately before the extension consumes the dot.
#[test]
fn libtorrent_decoder_decode_latin1_before_extension() {
    // Arrange
    let bytes = splice(b"song", I_ACUTE, b".flac");

    // Act
    let output = LibtorrentDecoder::decode(&bytes);

    // Assert
    assert_eq!(output, "song_lac");
}

/// A four-byte lead consumes its full claimed length of following bytes.
#[test]
fn libtorrent_decoder_decode_four_byte_lead() {
    // Arrange
    let bytes = splice(b"B", O_DIAERESIS, b"ser.txt");

    // Act
    let output = LibtorrentDecoder::decode(&bytes);

    // Assert
    assert_eq!(output, "B_.txt");
}

/// A lead byte with no valid sequence length skips a single byte.
#[test]
fn libtorrent_decoder_decode_invalid_lead_skips_one() {
    // Arrange
    let bytes = splice(b"", U_DIAERESIS, b"mlaut.txt");

    // Act
    let output = LibtorrentDecoder::decode(&bytes);

    // Assert
    assert_eq!(output, "_mlaut.txt");
}

/// Valid UTF-8 passes through unchanged.
#[test]
fn libtorrent_decoder_decode_valid_utf8() {
    // Arrange
    let bytes = "caf\u{e9}.flac".as_bytes();

    // Act
    let output = LibtorrentDecoder::decode(bytes);

    // Assert
    assert_eq!(output, "caf\u{e9}.flac");
}

/// A bad byte in the middle of the name leaves the extension intact.
#[test]
fn libtorrent_decoder_decode_midname_keeps_extension() {
    // Arrange
    let bytes = splice(b"Nac", I_ACUTE, b" Para.flac");

    // Act
    let output = LibtorrentDecoder::decode(&bytes);

    // Assert
    assert_eq!(output, "Nac_ara.flac");
}
