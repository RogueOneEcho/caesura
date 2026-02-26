use crate::prelude::*;

/// Actions that can fail in the batch module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum BatchAction {
    #[error("get unprocessed items")]
    GetUnprocessed,
    #[error("get queue item")]
    GetQueueItem,
    #[error("update queue item")]
    UpdateQueueItem,
    #[error("get source")]
    GetSource,
    #[error("serialize hook payload")]
    SerializeHookPayload,
    #[error("write hook payload")]
    WriteHookPayload,
    #[error("execute hook")]
    ExecuteHook,
}

/// Errors that can occur during batch processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum BatchError {
    #[error("unauthorized response received - this likely means the API key is invalid")]
    Unauthorized,
}
