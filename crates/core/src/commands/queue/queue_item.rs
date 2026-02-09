use crate::prelude::*;
use flat_db::Hash;
use lava_torrent::torrent::v1::Torrent;
use serde::{Deserialize, Serialize};

/// A source in the batch processing queue.
#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct QueueItem {
    /// Source name
    pub name: String,
    /// Torrent file path
    pub path: PathBuf,
    /// Source info hash
    pub hash: Hash<20>,
    /// Source indexer
    pub indexer: String,
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
    #[must_use]
    pub(crate) fn from_torrent(path: PathBuf, torrent: &Torrent) -> Self {
        let comment = torrent.comment().unwrap_or_default();
        let id = get_torrent_id_from_torrent_url(comment);
        let info_hash = torrent.info_hash();
        Self {
            name: torrent.name.clone(),
            path,
            hash: Hash::from_string(&info_hash).expect("torrent hash should be valid"),
            indexer: torrent.source().unwrap_or_default().to_lowercase(),
            id,
            ..Self::default()
        }
    }
}

impl Display for QueueItem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.name)
    }
}
