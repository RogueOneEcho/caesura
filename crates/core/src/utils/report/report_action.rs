use crate::prelude::*;

/// Actions that can fail in the report generation module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum ReportAction {
    #[error("create reports directory")]
    CreateDir,
    #[error("write report file")]
    WriteFile,
    #[error("read inspect output")]
    InspectFiles,
}
