use crate::prelude::*;

/// Verify content of a [`Source`] matches its torrent piece hashes.
#[injectable]
pub(crate) struct ContentVerifier {
    verify_options: Ref<VerifyOptions>,
    torrents: Ref<TorrentFileProvider>,
}

impl ContentVerifier {
    /// Verify the source content matches the torrent hash.
    ///
    /// - Skips entirely when `no_hash_check` is set
    /// - Downloads the source `.torrent`, caching it via [`TorrentFileProvider`]
    /// - Delegates piece hashing to [`TorrentVerifier`]
    ///
    /// Returns `Ok(None)` if the content matches or the check is skipped.
    /// Returns `Ok(Some(SourceIssue))` on a hash mismatch.
    pub(crate) async fn execute(
        &self,
        source: &Source,
    ) -> Result<Option<SourceIssue>, Failure<VerifyAction>> {
        if self.verify_options.no_hash_check {
            debug!("{} hash check due to settings", "Skipped".bold());
            return Ok(None);
        }
        trace!("Fetching torrent file for {}", source.torrent.id);
        let torrent_path = self
            .torrents
            .get(source.torrent.id)
            .await
            .map_err(Failure::wrap(VerifyAction::GetSourceTorrent))?;
        trace!(
            "{} torrent hash against {}",
            "Checking".bold(),
            source.directory.display()
        );
        TorrentVerifier::execute(&torrent_path, &source.directory)
            .await
            .map_err(Failure::wrap(VerifyAction::VerifyHash))
    }
}
