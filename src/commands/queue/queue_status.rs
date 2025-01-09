use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct QueueStatus {
    /// Did the queue command succeed?
    pub success: bool,
    /// Number of items added to the queue
    pub added: usize,
    /// Total number of items not added to the queue
    pub excluded: usize,
}
