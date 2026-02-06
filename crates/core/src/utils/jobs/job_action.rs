use crate::prelude::*;

/// Actions that can fail in the job module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum JobAction {
    #[error("transcode")]
    Transcode,
    #[error("generate spectrogram")]
    Spectrogram,
    #[error("process additional file")]
    Additional,
    #[error("execute task")]
    ExecuteTask,
}
