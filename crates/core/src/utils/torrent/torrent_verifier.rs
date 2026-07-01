//! Verify file contents match torrent piece hashes.

use crate::prelude::*;
use lava_torrent::torrent::v1::Torrent as LavaTorrent;

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
        let start = Instant::now();
        let torrent_file = torrent_file.to_path_buf();
        let directory = directory.to_path_buf();
        let result = spawn_blocking(move || verify(&torrent_file, &directory))
            .await
            .expect("torrent verify task should not panic");
        trace!(
            "{} torrent hash in {:.3}s",
            "Checked".bold(),
            start.elapsed().as_secs_f64()
        );
        result
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
    let mut hasher = match TorrentPieceHasher::open(&torrent, directory) {
        Ok(hasher) => hasher,
        Err(issue) => return Ok(Some(issue)),
    };
    trace!(
        "Hashing {} pieces from {}",
        torrent.pieces.len(),
        directory.display()
    );
    for (index, expected) in torrent.pieces.iter().enumerate() {
        match hasher.next() {
            None => return Ok(Some(SourceIssue::HashCheck { piece_index: index })),
            Some(Err(failure)) => return Err(failure),
            Some(Ok(digest)) => {
                if digest.as_slice() != expected.as_slice() {
                    return Ok(Some(SourceIssue::HashCheck { piece_index: index }));
                }
            }
        }
    }
    match hasher.next() {
        None => Ok(None),
        Some(Ok(_)) => Ok(Some(SourceIssue::ExcessContent)),
        Some(Err(failure)) => Err(failure),
    }
}
