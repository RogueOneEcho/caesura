#![allow(clippy::indexing_slicing)] // Indexing is safe in tests after length checks

use crate::testing_prelude::*;
use sha1::{Digest, Sha1};
use std::io::Cursor;

#[test]
fn torrent_piece_hasher_next_full_pieces() {
    // Arrange
    let data = vec![1_u8, 2, 3, 4, 5, 6, 7, 8];
    let mut hasher = TorrentPieceHasher::new(Box::new(Cursor::new(data.clone())), 4);

    // Act
    let first = hasher.next().expect("first piece").expect("no error");
    let second = hasher.next().expect("second piece").expect("no error");
    let third = hasher.next();

    // Assert
    assert_eq!(first.as_slice(), Sha1::digest(&data[0..4]).as_slice());
    assert_eq!(second.as_slice(), Sha1::digest(&data[4..8]).as_slice());
    assert!(third.is_none());
}

#[test]
fn torrent_piece_hasher_next_partial_final_piece() {
    // Arrange
    let data = vec![1_u8, 2, 3, 4, 5, 6];
    let mut hasher = TorrentPieceHasher::new(Box::new(Cursor::new(data.clone())), 4);

    // Act
    let first = hasher.next().expect("first piece").expect("no error");
    let second = hasher.next().expect("second piece").expect("no error");
    let third = hasher.next();

    // Assert
    assert_eq!(first.as_slice(), Sha1::digest(&data[0..4]).as_slice());
    assert_eq!(second.as_slice(), Sha1::digest(&data[4..6]).as_slice());
    assert!(third.is_none());
}

#[test]
fn torrent_piece_hasher_next_empty_stream() {
    let mut hasher = TorrentPieceHasher::new(Box::new(Cursor::new(Vec::new())), 4);
    assert!(hasher.next().is_none());
}

#[test]
fn torrent_piece_hasher_count_exact_multiple_pieces() {
    // Arrange
    let data = vec![0_u8; 12];
    let hasher = TorrentPieceHasher::new(Box::new(Cursor::new(data)), 4);

    // Act
    let count = hasher.count();

    // Assert
    assert_eq!(count, 3);
}
