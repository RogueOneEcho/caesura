use crate::commands::*;
use crate::utils::*;

use rogue_logging::Error;
use serde::{Deserialize, Serialize};

/// Result of an [`UploadCommand`] execution.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct UploadStatus {
    /// Did the upload command succeed?
    pub success: bool,
    /// Uploaded formats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<UploadFormatStatus>>,
    /// Time the upload completed.
    pub completed: TimeStamp,
    /// Error messages
    ///
    /// It is possible for [`UploadCommand`] to succeed while still having errors.
    /// For example `copy_transcode_to_content_dir` and `copy_torrent_to` are recoverable,
    /// so may error but the upload still proceeds successfully.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Error>>,
}

/// Status of a single format upload.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct UploadFormatStatus {
    /// Transcode format
    pub format: TargetFormat,
    /// ID of the upload
    pub id: u32,
}
