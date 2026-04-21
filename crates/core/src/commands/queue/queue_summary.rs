use crate::prelude::*;

/// Summary of items in the queue
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct QueueSummary {
    /// Total count
    pub total: usize,
    /// Count by known indexer
    pub indexer: BTreeMap<Indexer, usize>,
    /// Count of items with no known indexer
    pub indexer_unknown: usize,
    /// Awaiting verify count
    pub verify_none: usize,
    /// Successful verify count
    pub verify_verified_true: usize,
    /// Failed verify count
    pub verify_verified_false: usize,
    /// Awaiting spectrogram count
    pub spectrogram_none: usize,
    /// Successful spectrogram count
    pub spectrogram_success_true: usize,
    /// Failed spectrogram count
    pub spectrogram_success_false: usize,
    /// Awaiting transcode count
    pub transcode_none: usize,
    /// Successful transcode count
    pub transcode_success_true: usize,
    /// Failed transcode count
    pub transcode_success_false: usize,
    /// Awaiting upload count
    pub upload_none: usize,
    /// Successful uploads count
    pub upload_success_true: usize,
    /// Failed uploads count
    pub upload_success_false: usize,
}
