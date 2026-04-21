//! Verify file contents match torrent piece hashes.

use crate::prelude::*;
use lava_torrent::torrent::v1::Torrent as LavaTorrent;
use sha1::{Digest, Sha1};
use std::io::{Read, copy as io_copy, empty as io_empty};

/// Verify that files on disk match the piece hashes in a `.torrent` file.
pub struct TorrentVerifier;

impl TorrentVerifier {
    /// Verify content directory matches the torrent file.
    ///
    /// - Uses `spawn_blocking` because verification performs synchronous file I/O
    ///   and CPU-intensive piece hashing
    /// - Returns `Ok(None)` if all pieces match
    /// - Returns `Ok(Some(SourceIssue))` on validation failure
    pub async fn execute(
        torrent_file: &Path,
        directory: &Path,
    ) -> Result<Option<SourceIssue>, Failure<TorrentVerifyAction>> {
        let torrent_file = torrent_file.to_path_buf();
        let directory = directory.to_path_buf();
        spawn_blocking(move || verify(&torrent_file, &directory))
            .await
            .expect("torrent verify task should not panic")
    }
}

fn verify(
    torrent_file: &Path,
    directory: &Path,
) -> Result<Option<SourceIssue>, Failure<TorrentVerifyAction>> {
    let torrent = LavaTorrent::read_from_file(torrent_file).map_err(Failure::wrap_with_path(
        TorrentVerifyAction::ReadTorrent,
        torrent_file,
    ))?;
    let file_paths = get_file_paths(&torrent, directory);
    let mut stream = match open_content_stream(&file_paths) {
        Ok(stream) => stream,
        Err(issue) => return Ok(Some(issue)),
    };
    let piece_length = u64::try_from(torrent.piece_length).expect("piece length should fit in u64");
    let mut hasher = Sha1::new();
    for (index, expected) in torrent.pieces.iter().enumerate() {
        let mut piece = stream.by_ref().take(piece_length);
        let bytes_copied = io_copy(&mut piece, &mut hasher)
            .map_err(Failure::wrap(TorrentVerifyAction::HashContent))?;
        if bytes_copied == 0 {
            return Ok(Some(SourceIssue::HashCheck { piece_index: index }));
        }
        if hasher.finalize_reset().as_slice() != expected.as_slice() {
            return Ok(Some(SourceIssue::HashCheck { piece_index: index }));
        }
    }
    let mut extra = [0_u8; 1];
    if stream
        .read(&mut extra)
        .map_err(Failure::wrap(TorrentVerifyAction::HashContent))?
        > 0
    {
        return Ok(Some(SourceIssue::ExcessContent));
    }
    Ok(None)
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
