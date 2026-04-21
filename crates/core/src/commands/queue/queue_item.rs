use crate::prelude::*;
use lava_torrent::torrent::v1::Torrent;
use qbittorrent_api::get_torrents::Torrent as QbitTorrent;

/// A source in the batch processing queue.
#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct QueueItem {
    /// Source name
    pub name: String,
    /// Torrent file path
    pub path: PathBuf,
    /// Source info hash
    pub hash: Hash<20>,
    /// Source indexer, if known
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexer: Option<Indexer>,
    /// Source id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Verification status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyStatus>,
    /// Transcode status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectrogram: Option<SpectrogramStatus>,
    /// Transcode status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcode: Option<TranscodeStatus>,
    /// Upload status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<UploadStatus>,
}

impl QueueItem {
    /// Create a new [`QueueItem`] from a [`Torrent`].
    ///
    /// - Takes a reference to avoid moving the large `pieces` buffer
    /// - Sets `indexer` to [`None`] when the torrent has no `source` field or it is empty
    #[must_use]
    pub(crate) fn from_torrent(path: PathBuf, torrent: &Torrent) -> Self {
        let comment = torrent.comment().unwrap_or_default();
        let id = get_torrent_id_from_torrent_url(comment);
        let info_hash = torrent.info_hash();
        let indexer = torrent
            .source()
            .filter(|source| !source.is_empty())
            .map(Indexer::from);
        Self {
            name: torrent.name.clone(),
            path,
            hash: Hash::from_string(&info_hash).expect("torrent hash should be valid"),
            indexer,
            id,
            ..Self::default()
        }
    }

    /// Create a new [`QueueItem`] from a [`QbitTorrent`] API response.
    ///
    /// - Prefers `infohash_v1` over `hash` so hybrid torrents resolve to their v1 SHA-1
    ///   info hash. qBittorrent's `hash` field is the truncated SHA-256 v2 info hash for
    ///   hybrid and v2-only torrents
    /// - Derives `id` from the `torrentid` query parameter of the `comment` URL
    /// - Derives `indexer` from the host of the `comment` URL
    /// - Does not require reading or parsing `.torrent` files
    /// - Returns `None` if the chosen hash field cannot be parsed as a valid SHA-1 hex
    ///   string, which is the case for v2-only torrents
    #[must_use]
    pub(crate) fn from_qbit_torrent(torrent: &QbitTorrent) -> Option<Self> {
        let hash_string = torrent
            .infohash_v1
            .as_deref()
            .filter(|h| !h.is_empty())
            .unwrap_or(&torrent.hash);
        let hash = match Hash::from_string(hash_string) {
            Ok(hash) => hash,
            Err(error) => {
                warn!(
                    "{} torrent {}: invalid hash {}: {error}",
                    "Skipping".bold(),
                    torrent.name,
                    hash_string
                );
                return None;
            }
        };
        let comment = torrent.comment.as_deref().unwrap_or_default();
        let id = get_torrent_id_from_url(comment).ok();
        let indexer = get_indexer_from_url(comment);
        Some(Self {
            name: torrent.name.clone(),
            hash,
            indexer,
            id,
            ..Self::default()
        })
    }
}

impl Display for QueueItem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.name)
    }
}
