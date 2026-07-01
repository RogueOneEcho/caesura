//! Iterate SHA-1 digests of a torrent's content pieces.

use crate::prelude::*;
use lava_torrent::torrent::v1::Torrent as LavaTorrent;
use sha1::{Digest, Sha1};
use std::io::{Read, copy as io_copy, empty as io_empty};

/// Yield SHA-1 digests of successive torrent pieces from a content byte stream.
pub(crate) struct TorrentPieceHasher {
    stream: Box<dyn Read>,
    piece_length: u64,
    hasher: Sha1,
}

impl TorrentPieceHasher {
    /// Create a new [`TorrentPieceHasher`] over the content stream.
    ///
    /// - `piece_length` is the torrent's piece length in bytes
    pub(crate) fn new(stream: Box<dyn Read>, piece_length: u64) -> Self {
        Self {
            stream,
            piece_length,
            hasher: Sha1::new(),
        }
    }

    /// Create a new [`TorrentPieceHasher`] by opening the torrent's content files.
    ///
    /// - Resolves content file paths from the torrent metadata under `directory`
    /// - Opens and chains those files into one contiguous stream
    ///
    /// Errors returned as [`SourceIssue`] when a file is missing or cannot be opened.
    pub(crate) fn open(torrent: &LavaTorrent, directory: &Path) -> Result<Self, SourceIssue> {
        let paths = get_file_paths(torrent, directory);
        trace!("Opening {} content files", paths.len());
        let stream = open_content_stream(&paths)?;
        let piece_length =
            u64::try_from(torrent.piece_length).expect("piece length should fit in u64");
        Ok(Self::new(stream, piece_length))
    }
}

impl Iterator for TorrentPieceHasher {
    type Item = Result<[u8; 20], Failure<TorrentVerifyAction>>;

    /// Hash the next piece-sized chunk of the content stream.
    ///
    /// - Returns [`None`] once the stream is exhausted
    /// - Returns the digest of a shorter final piece when content ends mid-piece
    /// - Returns an error if reading the content stream fails
    fn next(&mut self) -> Option<Self::Item> {
        let mut piece = self.stream.by_ref().take(self.piece_length);
        match io_copy(&mut piece, &mut self.hasher) {
            Ok(0) => None,
            Ok(_) => {
                let mut digest = [0_u8; 20];
                digest.copy_from_slice(self.hasher.finalize_reset().as_slice());
                Some(Ok(digest))
            }
            Err(error) => Some(Err(Failure::new(TorrentVerifyAction::HashContent, error))),
        }
    }
}

/// Open all content files and chain them into a single contiguous byte stream.
///
/// Torrent pieces span across file boundaries, so individual files cannot be
/// hashed independently. Chaining the files into one stream lets the caller
/// read piece-sized chunks with [`Read::take`] without manually tracking
/// which file it's in or how many bytes remain until a piece boundary.
///
/// Opening all files upfront also detects missing or inaccessible files before
/// any hashing begins, rather than discovering them mid-stream.
///
/// Memory usage is minimal: each [`chain`](Read::chain) is a zero-cost wrapper
/// that delegates reads to the underlying file handles. No file content is
/// buffered beyond what [`io_copy`] uses internally (8 KB).
fn open_content_stream(paths: &[PathBuf]) -> Result<Box<dyn Read>, SourceIssue> {
    let mut stream: Box<dyn Read> = Box::new(io_empty());
    for path in paths {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(SourceIssue::MissingFile { path: path.clone() });
            }
            Err(e) => {
                return Err(SourceIssue::OpenFile {
                    path: path.clone(),
                    error: e.to_string(),
                });
            }
        };
        stream = Box::new(stream.chain(file));
    }
    Ok(stream)
}

/// Build the ordered list of file paths from the torrent metadata.
fn get_file_paths(torrent: &LavaTorrent, directory: &Path) -> Vec<PathBuf> {
    match &torrent.files {
        Some(files) => files.iter().map(|f| directory.join(&f.path)).collect(),
        None => vec![directory.join(&torrent.name)],
    }
}
