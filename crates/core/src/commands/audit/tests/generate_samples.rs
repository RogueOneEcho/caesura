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
    assert_eq!(samples.len(), 24);
}

/// The `samples/audit` directory relative to the workspace root.
fn samples_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/audit")
}

/// The demo samples as `(file_name, bencode_bytes)` pairs.
fn samples() -> Vec<(String, Vec<u8>)> {
    vec![
        (
            "non-utf8-accents-in-tracks.torrent".to_owned(),
            non_utf8_accents_in_tracks(),
        ),
        (
            "non-utf8-accent-in-folder.torrent".to_owned(),
            non_utf8_accent_in_folder(),
        ),
        (
            "non-breaking-space-in-tracks.torrent".to_owned(),
            non_breaking_space_in_tracks(),
        ),
        (
            "zero-width-space-in-track.torrent".to_owned(),
            zero_width_space_in_track(),
        ),
        (
            "ltr-mark-in-folder.torrent".to_owned(),
            ltr_mark_in_folder(),
        ),
        ("ltr-mark-in-track.torrent".to_owned(), ltr_mark_in_track()),
        (
            "backslash-in-track.torrent".to_owned(),
            backslash_in_track(),
        ),
        (
            "backslash-in-folder.torrent".to_owned(),
            backslash_in_folder(),
        ),
        (
            "decomposed-accents-in-tracks.torrent".to_owned(),
            decomposed_accents_in_tracks(),
        ),
        (
            "decomposed-accent-in-folder.torrent".to_owned(),
            decomposed_accent_in_folder(),
        ),
        (
            "non-utf8-lost-extension-e.torrent".to_owned(),
            non_utf8_lost_extension_e(),
        ),
        (
            "non-utf8-lost-extension-a.torrent".to_owned(),
            non_utf8_lost_extension_a(),
        ),
        (
            "single-file-no-files-list-red.torrent".to_owned(),
            single_file_no_files_list_red(),
        ),
        (
            "single-file-no-files-list-ops.torrent".to_owned(),
            single_file_no_files_list_ops(),
        ),
        (
            "forward-slash-in-track.torrent".to_owned(),
            forward_slash_in_track(),
        ),
        (
            "trailing-slash-in-folder.torrent".to_owned(),
            trailing_slash_in_folder(),
        ),
        (
            "trailing-backslash-in-folder.torrent".to_owned(),
            trailing_backslash_in_folder(),
        ),
        (
            "trailing-division-slash-in-folder.torrent".to_owned(),
            trailing_division_slash_in_folder(),
        ),
        (
            "trailing-fullwidth-slash-in-folder.torrent".to_owned(),
            trailing_fullwidth_slash_in_folder(),
        ),
        (
            "trailing-overlong-slash-in-folder.torrent".to_owned(),
            trailing_overlong_slash_in_folder(),
        ),
        (
            "necessary-mark-beside-rtl-in-folder.torrent".to_owned(),
            necessary_mark_beside_rtl_in_folder(),
        ),
        ("isolate-in-track.torrent".to_owned(), isolate_in_track()),
        (
            "rtl-override-in-track.torrent".to_owned(),
            rtl_override_in_track(),
        ),
        (
            "rtl-mark-in-folder.torrent".to_owned(),
            rtl_mark_in_folder(),
        ),
    ]
}

/// A windows-1252 `è` and `î` in track names.
fn non_utf8_accents_in_tracks() -> Vec<u8> {
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
fn non_utf8_accent_in_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_002,
        splice(b"Test Artist 2 - Sample Album ", 0xE4, b" [FLAC]"),
        vec![file("Test Artist 2 - Sample Track.flac")],
    )
}

/// A non-breaking space in track names.
fn non_breaking_space_in_tracks() -> Vec<u8> {
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
fn zero_width_space_in_track() -> Vec<u8> {
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
fn ltr_mark_in_folder() -> Vec<u8> {
    multi(
        "RED",
        100_005,
        "Test Artist 5 - Sample Album \u{200E}[FLAC]",
        vec![file("Test Artist 5 - Sample Track.flac")],
    )
}

/// A left-to-right mark in a track name.
fn ltr_mark_in_track() -> Vec<u8> {
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

/// A left-to-right mark beside right-to-left script in the folder name.
///
/// `U+0623 U+0644 U+0628 U+0648 U+0645` is Arabic, so the trailing `U+200E`
/// legitimately orders the Latin tag and is not flagged as unnecessary. It is
/// still stripped by libtorrent, so the on-disk name diverges.
fn necessary_mark_beside_rtl_in_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_021,
        "Test Artist 21 - \u{0623}\u{0644}\u{0628}\u{0648}\u{0645}\u{200E} [FLAC]",
        vec![file("Test Artist 21 - Sample Track.flac")],
    )
}

/// A directional isolate wrapping a segment of a track name.
///
/// `U+2066` and `U+2069` are isolate controls with no effect beside Latin text.
fn isolate_in_track() -> Vec<u8> {
    multi(
        "RED",
        100_022,
        "Test Artist 22 - Sample Album 22 [FLAC]",
        vec![
            file("01 - Sample Track \u{2066}One\u{2069}.flac"),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A right-to-left override in a track name.
///
/// `U+202E` visually reverses the following text, the classic trojan-source
/// spoof. It has no right-to-left script to order and is stripped by libtorrent.
fn rtl_override_in_track() -> Vec<u8> {
    multi(
        "OPS",
        100_023,
        "Test Artist 23 - Sample Album 23 [FLAC]",
        vec![
            file("01 - Sample Track \u{202E}One.flac"),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A right-to-left mark in the folder name.
///
/// `U+200F` has no right-to-left script to order beside the Latin name, so it is
/// inert and unnecessary.
fn rtl_mark_in_folder() -> Vec<u8> {
    multi(
        "RED",
        100_024,
        "Test Artist 24 - Sample Album 24 \u{200F}[FLAC]",
        vec![file("Test Artist 24 - Sample Track.flac")],
    )
}

/// A backslash in a track name.
fn backslash_in_track() -> Vec<u8> {
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
fn backslash_in_folder() -> Vec<u8> {
    multi(
        "RED",
        100_008,
        "Test Artist 8 - Sample Album \\ [FLAC]",
        vec![file("Test Artist 8 - Sample Track.flac")],
    )
}

/// A forward slash in a track name.
fn forward_slash_in_track() -> Vec<u8> {
    multi(
        "OPS",
        100_015,
        "Test Artist 15 - Sample Album 15 [FLAC]",
        vec![
            file("01 - Sample Track/One.flac"),
            file("02 - Sample Track.flac"),
        ],
    )
}

/// A trailing forward slash on the folder name.
fn trailing_slash_in_folder() -> Vec<u8> {
    multi(
        "RED",
        100_016,
        "Test Artist 16 - Sample Album 16 [FLAC]/",
        vec![file("Test Artist 16 - Sample Track.flac")],
    )
}

/// A trailing backslash on the folder name.
fn trailing_backslash_in_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_017,
        "Test Artist 17 - Sample Album 17 [FLAC]\\",
        vec![file("Test Artist 17 - Sample Track.flac")],
    )
}

/// A trailing division slash on the folder name.
///
/// `U+2215` is the Windows lookalike substituted for a forward slash.
fn trailing_division_slash_in_folder() -> Vec<u8> {
    multi(
        "RED",
        100_018,
        "Test Artist 18 - Sample Album 18 [FLAC]\u{2215}",
        vec![file("Test Artist 18 - Sample Track.flac")],
    )
}

/// A trailing fullwidth solidus on the folder name.
///
/// `U+FF0F` decomposes to a plain forward slash under NFKC normalization.
fn trailing_fullwidth_slash_in_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_019,
        "Test Artist 19 - Sample Album 19 [FLAC]\u{FF0F}",
        vec![file("Test Artist 19 - Sample Track.flac")],
    )
}

/// A trailing overlong-encoded forward slash on the folder name.
///
/// The bytes `0xC0 0xAF` are an overlong UTF-8 encoding a lenient decoder
/// collapses to a plain forward slash.
fn trailing_overlong_slash_in_folder() -> Vec<u8> {
    let mut name = b"Test Artist 20 - Sample Album 20 [FLAC]".to_vec();
    name.extend_from_slice(&[0xC0, 0xAF]);
    multi(
        "RED",
        100_020,
        name,
        vec![file("Test Artist 20 - Sample Track.flac")],
    )
}

/// A decomposed (non-NFC) `é` in track names.
fn decomposed_accents_in_tracks() -> Vec<u8> {
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
fn decomposed_accent_in_folder() -> Vec<u8> {
    multi(
        "OPS",
        100_010,
        "Test Artist 10 - Sample Album No\u{0308}rd [FLAC]",
        vec![file("Test Artist 10 - Sample Track No\u{0308}rd.flac")],
    )
}

/// A windows-1252 `é` immediately before the extension.
fn non_utf8_lost_extension_e() -> Vec<u8> {
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
fn non_utf8_lost_extension_a() -> Vec<u8> {
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

/// A single-file RED torrent with no `files` list.
fn single_file_no_files_list_red() -> Vec<u8> {
    single("RED", 100_013, "Test Artist 13 - Sample Single Track.flac")
}

/// A single-file OPS torrent with no `files` list.
fn single_file_no_files_list_ops() -> Vec<u8> {
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
