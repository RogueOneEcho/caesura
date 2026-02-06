//! Action types for external process execution errors.

use crate::prelude::*;

/// Action that failed during external process execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum ProcessAction {
    /// Process failed to start (e.g., not found, permission denied).
    #[error("start process")]
    Start,
    /// Process ran but exited with non-zero status.
    #[error("execute process")]
    Execute,
}
