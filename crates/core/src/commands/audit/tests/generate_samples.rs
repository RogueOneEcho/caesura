//! Generator for the `audit` command demo samples.
//!
//! Writes a batch of `.torrent` files with clearly synthetic artists, albums,
//! and tracks. Only the fault patterns reproduce real-world torrents. Each
//! carries a placeholder announce URL so no tracker passkey is embedded.

use crate::testing_prelude::*;
use std::fs::create_dir_all;
use std::path::PathBuf;

/// Placeholder announce URL, so samples carry no real tracker passkey.
const ANNOUNCE: &str = "https://tracker.invalid/announce";

/// Fixed piece length for the sample torrents.
const PIECE_LENGTH: i64 = 16384;

/// Write the `audit` demo samples to `samples/audit`.
///
/// Ignored by default as it writes files rather than asserting behavior.
/// Run with `cargo test -p caesura generate_audit_samples -- --ignored`.
#[test]
#[ignore = "generates demo sample files, not a test"]
fn generate_audit_samples() {
    // Arrange
    let directory = samples_directory();
    create_dir_all(&directory).expect("should create samples directory");
    let samples = samples();

    // Act
    for (file_name, bytes) in &samples {
        write(directory.join(file_name), bytes).expect("should write sample torrent");
    }

    // Assert
    assert_eq!(samples.len(), 14);
}

/// The `samples/audit` directory relative to the workspace root.
fn samples_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/audit")
}

/// The demo samples as `(file_name, bencode_bytes)` pairs.
fn samples() -> Vec<(String, Vec<u8>)> {
    vec![
        ("non-utf8-1.torrent".to_owned(), non_utf8_accents()),
        ("non-utf8-2.torrent".to_owned(), non_utf8_folder()),
        ("invisible-1.torrent".to_owned(), invisible_nbsp()),
        ("invisible-2.torrent".to_owned(), invisible_zwsp()),
        ("libtorrent-1.torrent".to_owned(), libtorrent_folder()),
        ("libtorrent-2.torrent".to_owned(), libtorrent_track()),
        ("unsafe-1.torrent".to_owned(), unsafe_track()),
        ("unsafe-2.torrent".to_owned(), unsafe_folder()),
        ("decomposed-1.torrent".to_owned(), decomposed_tracks()),
        ("decomposed-2.torrent".to_owned(), decomposed_folder()),
        ("lost-extension-1.torrent".to_owned(), lost_extension_e()),
        ("lost-extension-2.torrent".to_owned(), lost_extension_a()),
        ("single-file-1.torrent".to_owned(), single_file_album()),
        ("single-file-2.torrent".to_owned(), single_file_single()),
    ]
}

/// A windows-1252 `è` and `î` in track names.
fn non_utf8_accents() -> Vec<u8> {
    multi(
        "RED",
        100_001,
        "Test Artist 1 - Sample Album 1 [FLAC]",
        vec![
            file(splice(b"01 - Sample Track ", 0xE8, b" One.flac")),
            file(splice(b"02 - Sample Track ", 0xEE, b" Two.flac")),
        ],
    )
}

/// A windows-1252 `ä` in the folder name.
fn non_utf8_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_002,
        splice(b"Test Artist 2 - Sample Album ", 0xE4, b" [FLAC]"),
        vec![file("Test Artist 2 - Sample Track.flac")],
    )
}

/// A non-breaking space in track names.
fn invisible_nbsp() -> Vec<u8> {
    multi(
        "RED",
        100_003,
        "Test Artist 3 - Sample Album 3 [FLAC]",
        vec![
            file("01 - Sample Track.flac"),
            file("02 - Sample Track with inv\u{00A0}isible.flac"),
            file("03 - Sample Track with another\u{00A0}.flac"),
        ],
    )
}

/// A zero-width space in a track name.
fn invisible_zwsp() -> Vec<u8> {
    multi(
        "OPS",
        100_004,
        "Test Artist 4 - Sample Album 4 [FLAC]",
        vec![
            file("01 - Sample Track with invisible\u{200B}.flac"),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A left-to-right mark in the folder name.
fn libtorrent_folder() -> Vec<u8> {
    multi(
        "RED",
        100_005,
        "Test Artist 5 - Sample Album \u{200E}[FLAC]",
        vec![file("Test Artist 5 - Sample Track.flac")],
    )
}

/// A left-to-right mark in a track name.
fn libtorrent_track() -> Vec<u8> {
    multi(
        "OPS",
        100_006,
        "Test Artist 6 - Sample Album 6 [FLAC]",
        vec![
            file("00 - Sample Playlist \u{200E}.m3u"),
            file("01 - Sample Track.flac"),
        ],
    )
}

/// A backslash in a track name.
fn unsafe_track() -> Vec<u8> {
    multi(
        "RED",
        100_007,
        "Test Artist 7 - Sample Album 7 [FLAC]",
        vec![
            file("01 - Sample Track\\.flac"),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A backslash in the folder name.
fn unsafe_folder() -> Vec<u8> {
    multi(
        "RED",
        100_008,
        "Test Artist 8 - Sample Album \\ [FLAC]",
        vec![file("Test Artist 8 - Sample Track.flac")],
    )
}

/// A decomposed (non-NFC) `é` in track names.
fn decomposed_tracks() -> Vec<u8> {
    multi(
        "RED",
        100_009,
        "Test Artist 9 - Sample Album 9 [FLAC]",
        vec![
            file("01 - Sample Track Sce\u{0301}ne.flac"),
            file("02 - Sample Track Re\u{0301}sume.flac"),
        ],
    )
}

/// A decomposed (non-NFC) `ö` in the folder and track names.
fn decomposed_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_010,
        "Test Artist 10 - Sample Album No\u{0308}rd [FLAC]",
        vec![file("Test Artist 10 - Sample Track No\u{0308}rd.flac")],
    )
}

/// A windows-1252 `é` immediately before the extension.
fn lost_extension_e() -> Vec<u8> {
    multi(
        "RED",
        100_011,
        "Test Artist 11 - Sample Album 11 [FLAC]",
        vec![
            file(splice(b"01 - Sample Track", 0xE9, b".flac")),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A windows-1252 `ä` immediately before the extension.
fn lost_extension_a() -> Vec<u8> {
    multi(
        "OPS",
        100_012,
        "Test Artist 12 - Sample Album 12 [FLAC]",
        vec![
            file(splice(b"01 - Sample Track", 0xE4, b".flac")),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A single-file torrent with no `files` list.
fn single_file_album() -> Vec<u8> {
    single("RED", 100_013, "Test Artist 13 - Sample Single Track.flac")
}

/// A single-file torrent with no `files` list.
fn single_file_single() -> Vec<u8> {
    single("OPS", 100_014, "Test Artist 14 - Sample Single Track.flac")
}

/// A file entry with a single-component `path` of `component`.
fn file(component: impl Into<RawString>) -> Vec<RawString> {
    vec![component.into()]
}

/// A multi-file torrent named `name` with one file per `files` path.
fn multi(source: &str, id: u32, name: impl Into<RawString>, files: Vec<Vec<RawString>>) -> Vec<u8> {
    let files = files.into_iter().map(TorrentBuilder::file).collect();
    let info = TorrentBuilder::new()
        .with_dictionaries("files", files)
        .with_string("name", name)
        .with_integer("piece length", PIECE_LENGTH)
        .with_string("pieces", vec![0_u8; 20])
        .with_string("source", source);
    wrap(source, id, info)
}

/// A single-file torrent named `name` with no `files` list.
fn single(source: &str, id: u32, name: impl Into<RawString>) -> Vec<u8> {
    let info = TorrentBuilder::new()
        .with_integer("length", PIECE_LENGTH)
        .with_string("name", name)
        .with_integer("piece length", PIECE_LENGTH)
        .with_string("pieces", vec![0_u8; 20])
        .with_string("source", source);
    wrap(source, id, info)
}

/// Wrap `info` in a top-level dictionary with a placeholder announce and comment URL.
fn wrap(source: &str, id: u32, info: TorrentBuilder) -> Vec<u8> {
    TorrentBuilder::new()
        .with_string("announce", ANNOUNCE)
        .with_string("comment", comment(source, id))
        .with_dictionary("info", info)
        .build()
}

/// A tracker `comment` URL for `source` and torrent `id`.
fn comment(source: &str, id: u32) -> String {
    let host = if source == "OPS" { OPS_URL } else { RED_URL };
    format!("{host}/torrents.php?id=1&torrentid={id}#torrent{id}")
}
